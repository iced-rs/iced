use std::fmt::Display;
use std::sync::{Arc, LazyLock};

use async_openai::Client as OpenAiClient;
use async_openai::config::OpenAIConfig;
use async_openai::types::CreateEmbeddingRequestArgs;
use iced::theme::{Custom, Palette};
use iced::widget::{button, row, scrollable, text_editor, text_input};
use mistralai_client::v1::client::Client as MistralClient;
use serde::Deserialize;
use surrealdb::engine::any::{Any, connect};
use surrealdb::{RecordId, Surreal, Value};

use iced::widget::{column, text};
use iced::{Center, Element, Theme, color};
use tokio::runtime::Runtime;

use mistralai_client::v1::constants::EmbedModel::MistralEmbed;

static OPENAI_API_KEY: LazyLock<String> =
    LazyLock::new(|| std::env::var("OPENAI_API_KEY").unwrap_or("NONE".to_string()));

static MISTRAL_API_KEY: LazyLock<String> =
    LazyLock::new(|| std::env::var("MISTRAL_API_KEY").unwrap_or("NONE".to_string()));

static OPENAI_CLIENT: LazyLock<OpenAiClient<OpenAIConfig>> = LazyLock::new(|| {
    let config = OpenAIConfig::new().with_api_key(&*OPENAI_API_KEY);
    OpenAiClient::with_config(config)
});

static MISTRAL_CLIENT: LazyLock<MistralClient> =
    LazyLock::new(|| MistralClient::new(Some(MISTRAL_API_KEY.clone()), None, None, None).unwrap());

trait StringOutput {
    fn output(self) -> String;
}

impl StringOutput for Result<String, String> {
    fn output(self) -> String {
        match self {
            Ok(o) => o,
            Err(e) => e,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct PageContent {
    title: String,
    extract: String,
}

fn page_for(page: &str) -> String {
    format!("http://en.wikipedia.org/api/rest_v1/page/summary/{page}")
}

fn init() -> &'static str {
    r#"
    DEFINE NAMESPACE ns;
    DEFINE DATABASE db;
    USE NS ns;
    USE DB db;
    DEFINE FIELD extract ON document TYPE string;
    DEFINE FIELD title ON document TYPE string;
    DEFINE FIELD mistral_embedding ON document TYPE option<array<float>> DEFAULT [];
    DEFINE FIELD openai_embedding ON document TYPE option<array<float>> DEFAULT [];
    DEFINE ANALYZER en_analyzer TOKENIZERS class FILTERS lowercase,edgengram(3,10);
    DEFINE INDEX en_extract ON document FIELDS extract SEARCH ANALYZER en_analyzer BM25 HIGHLIGHTS;
    DEFINE INDEX en_title ON document FIELDS title SEARCH ANALYZER en_analyzer BM25 HIGHLIGHTS;

    DEFINE TABLE link TYPE RELATION IN document OUT document ENFORCED;

    DEFINE INDEX only_one_link ON link FIELDS in,out UNIQUE;"#
}

struct App {
    rt: Option<Runtime>,
    db: Surreal<Any>,
    app_output: String,
    query_content: text_editor::Content,
    document_content: text_editor::Content,
    link_content: text_editor::Content,
    openai_doc_search: text_editor::Content,
    mistral_doc_search: text_editor::Content,
    fts_text: text_editor::Content,
    seelinks_text: text_editor::Content,
}

impl Default for App {
    fn default() -> Self {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let db = rt.block_on(async {
            let db = connect("memory").await.unwrap();
            db.use_db("db").await.unwrap();
            db.use_ns("ns").await.unwrap();
            db.query(init()).await.unwrap();
            db
        });

        Self {
            rt: Some(rt),
            db,
            app_output: Default::default(),
            query_content: Default::default(),
            document_content: Default::default(),
            link_content: Default::default(),
            openai_doc_search: Default::default(),
            fts_text: Default::default(),
            mistral_doc_search: Default::default(),
            seelinks_text: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Query,
    QueryContent(text_editor::Action),
    InsertDocuments,
    InsertDocumentsContent(text_editor::Action),
    LinkDocuments,
    LinkDocumentsContent(text_editor::Action),
    OpenAiSimilaritySearch,
    OpenAiSimilaritySearchContent(text_editor::Action),
    MistralSimilaritySearch,
    MistralSimilaritySearchContent(text_editor::Action),
    Fts,
    FtsContent(text_editor::Action),
    TryLink,
    SeeDocs,
    SeeLinks,
    SeeLinksContent(text_editor::Action),
    AddOpenAi,
    AddMistral,
}

async fn get_openai_embeddings(content: Vec<PageContent>) -> Result<Vec<Vec<f32>>, String> {
    let extracts = content
        .into_iter()
        .map(|v| v.extract)
        .collect::<Vec<String>>();
    // Get the OpenAI embeddings
    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-3-small")
        .input(extracts)
        .dimensions(1536u32)
        .build()
        .map_err(|e| e.to_string())?;
    match OPENAI_CLIENT.embeddings().create(request).await {
        Ok(res) => Ok(res
            .data
            .into_iter()
            .map(|v| v.embedding)
            .collect::<Vec<Vec<f32>>>()),
        Err(e) => Err(e.to_string()),
    }
}

async fn get_mistral_embeddings(content: Vec<PageContent>) -> Result<Vec<Vec<f32>>, String> {
    let extracts = content
        .into_iter()
        .map(|v| v.extract)
        .collect::<Vec<String>>();
    // Get the Mistral embeddings
    match MISTRAL_CLIENT
        .embeddings_async(MistralEmbed, extracts, None)
        .await
    {
        Ok(res) => Ok(res
            .data
            .into_iter()
            .map(|d| d.embedding)
            .collect::<Vec<Vec<f32>>>()),
        Err(e) => Err(e.to_string()),
    }
}

fn get_possible_links(title: &str, content: &str) -> Vec<String> {
    content
        .split_whitespace()
        .filter(|word| matches!(word.chars().next(), Some(c) if c.is_uppercase()))
        .filter_map(|word| {
            let only_alpha = word
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>();
            // Keep long words
            if only_alpha.chars().count() >= 3 && only_alpha != title {
                Some(only_alpha)
            } else {
                None
            }
        })
        //.flatten()
        .collect::<Vec<String>>()
}

#[derive(Deserialize, Debug)]
struct LinkOutput {
    r#in: RecordId,
    out: RecordId,
}

impl Display for LinkOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "in: {} out: {}", self.r#in.key(), self.out.key())
    }
}

impl App {
    async fn try_to_link(&self) -> Result<String, String> {
        let mut response = self
            .db
            .query("SELECT * FROM document")
            .await
            .map_err(|e| e.to_string())?;
        let unlinked_docs = response
            .take::<Vec<PageContent>>(0)
            .map_err(|e| format!("{e:?}"))?;
        let mut output = String::from("Docs linked: ");
        for doc in unlinked_docs {
            let possible_links = get_possible_links(&doc.title, &doc.extract);
            for link in possible_links {
                let first = RecordId::from(("document", &doc.title));
                let second = RecordId::from(("document", &link));
                if let Ok(mut o) = self
                    .db
                    .query("RELATE $first->link->$second")
                    .bind(("first", first))
                    .bind(("second", second))
                    .await
                {
                    if let Ok(Some(o)) = &o.take::<Option<LinkOutput>>(0) {
                        output += "\n";
                        output += &o.to_string();
                    }
                }
            }
        }
        Ok(output)
    }

    async fn ai_similarity_search(
        &self,
        doc: String,
        field_name: String,
    ) -> Result<Value, surrealdb::Error> {
        let doc = doc.trim().to_owned();
        let field_name = field_name.trim().to_owned();

        let mut current_doc = self
            .db
            // Grab just the embeds field from a document
            .query(format!("type::thing('document', '{doc}').{field_name};"))
            .await?;
        let embeds: Value = current_doc.take(0)?;

        let mut similar = self
            .db
            .query(format!(
                "(SELECT 
    (extract.slice(0, 50) + '...') AS extract,
    title,
    vector::distance::knn() AS distance
        FROM document
        WHERE {field_name} <|4,COSINE|> $embeds
        ORDER BY distance).filter(|$t| $t.distance > 0.0001);",
            ))
            .bind(("embeds", embeds))
            .await?;
        similar.take::<Value>(0)
    }

    async fn add_openai(&self) -> Result<String, String> {
        let no_open_id: Vec<PageContent> = self
            .db
            .query("SELECT title, extract FROM document WHERE !openai_embedding")
            .await
            .map_err(|e| e.to_string())?
            .take(0)
            .map_err(|e| e.to_string())?;
        if !no_open_id.is_empty() {
            let embeddings = get_openai_embeddings(no_open_id.clone())
                .await
                .map_err(|e| e.to_string())?;
            let zipped = no_open_id.into_iter().zip(embeddings.into_iter());

            let mut results = String::from("Embeddings added for:");
            for (one, two) in zipped {
                let mut res = self
                    .db
                    .query(
                        "UPDATE type::thing('document', $title)
            SET openai_embedding = $embeds",
                    )
                    .bind(("title", one.title))
                    .bind(("embeds", two))
                    .await
                    .map_err(|e| e.to_string())?;
                if let Ok(Some(v)) = res.take::<Option<PageContent>>(0) {
                    results.push('\n');
                    results.push_str(&v.title);
                }
            }
            Ok(results)
        } else {
            Err(String::from("No documents found to update"))
        }
    }

    async fn add_mistral(&self) -> Result<String, String> {
        let no_mistral_id: Vec<PageContent> = self
            .db
            .query("SELECT title, extract FROM document WHERE !mistral_embedding")
            .await
            .map_err(|e| e.to_string())?
            .take(0)
            .map_err(|e| e.to_string())?;
        if !no_mistral_id.is_empty() {
            let embeddings = get_mistral_embeddings(no_mistral_id.clone())
                .await
                .map_err(|e| e.to_string())?;
            let zipped = no_mistral_id.into_iter().zip(embeddings.into_iter());

            let mut results = String::from("Embeddings added for:");
            for (one, two) in zipped {
                let mut res = self
                    .db
                    .query(
                        "UPDATE type::thing('document', $title)
            SET mistral_embedding = $embeds",
                    )
                    .bind(("title", one.title))
                    .bind(("embeds", two))
                    .await
                    .map_err(|e| e.to_string())?;
                match res.take::<Option<PageContent>>(0) {
                    Ok(Some(v)) => {
                        results.push('\n');
                        results.push_str(&v.title);
                    }
                    Ok(None) => return Err("No PageContent found".to_string()),
                    Err(e) => return Ok(e.to_string()),
                }
            }
            Ok(results)
        } else {
            Err(String::from("No documents found to update"))
        }
    }

    async fn insert_documents(&self, page_names: String) -> Result<String, String> {
        // Get each page name separated by a comma and add _ in between words
        let page_names = page_names
            .trim()
            .split(",")
            .map(|p| p.split_whitespace().collect::<Vec<&str>>().join("_"));
        let mut result = String::new();

        let results = page_names
            .map(|page| {
                std::thread::spawn(move || {
                    let url = page_for(&page);
                    let res = ureq::get(url).call();
                    match res {
                        Ok(mut o) => o.body_mut().read_to_string().unwrap(),
                        Err(_) => format!("No page {page} found"),
                    }
                })
            })
            .collect::<Vec<_>>();

        for res in results {
            let s = res.join().unwrap();
            let mut content: PageContent = match serde_json::from_str(&s) {
                Ok(content) => content,
                Err(_) => return Err(s),
            };
            // Add an underscore again as response from Wikipedia won't have it
            content.title = content
                .title
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("_");
            result.push('\n');
            result.push_str(&self.add_document(content).await);
        }
        Ok(result)
    }

    async fn add_document(&self, content: PageContent) -> String {
        let doc = RecordId::from_table_key("document", &content.title);

        let res = self
            .db
            .query("CREATE ONLY $doc SET title = $title, extract = $extract;")
            .bind(("doc", doc))
            .bind(("title", content.title))
            .bind(("extract", content.extract));
        match res.await {
            Ok(mut r) => match r.take::<Option<PageContent>>(0) {
                Ok(Some(good)) => format!("{good:?}"),
                Ok(None) => "No PageContent found".to_string(),
                Err(e) => e.to_string(),
            },
            Err(e) => e.to_string(),
        }
    }

    fn update(&mut self, message: Message) {
        use Message as M;

        let rt = self.rt.take().unwrap();

        rt.block_on(async {
            match message {
                M::Query => self.app_output = self.raw_query(&self.query_content.text()).await,
                M::InsertDocuments => {
                    let content = self
                        .insert_documents(self.document_content.text())
                        .await
                        .output();

                    self.app_output = format!("Add article result: {content}");
                }
                M::LinkDocuments => {
                    self.app_output = self.link_documents(self.link_content.text()).await
                }

                M::OpenAiSimilaritySearch => {
                    self.app_output = match self
                        .ai_similarity_search(
                            self.openai_doc_search.text(),
                            "openai_embedding".to_string(),
                        )
                        .await
                    {
                        Ok(o) => o.to_string(),
                        Err(e) => e.to_string(),
                    }
                }
                M::MistralSimilaritySearch => {
                    self.app_output = match self
                        .ai_similarity_search(
                            self.mistral_doc_search.text(),
                            "mistral_embedding".to_string(),
                        )
                        .await
                    {
                        Ok(o) => o.to_string(),
                        Err(e) => e.to_string(),
                    }
                }
                M::TryLink => self.app_output = self.try_to_link().await.output(),
                M::Fts => self.app_output = self.fts_search(self.fts_text.text()).await.output(),
                M::SeeDocs => self.app_output = self.see_docs().await,
                M::SeeLinks => self.app_output = self.linked_docs(self.seelinks_text.text()).await,
                M::AddOpenAi => self.app_output = self.add_openai().await.output(),
                M::AddMistral => self.app_output = self.add_mistral().await.output(),
                // Text windows
                M::QueryContent(action) => self.query_content.perform(action),
                M::InsertDocumentsContent(action) => self.document_content.perform(action),
                M::LinkDocumentsContent(action) => self.link_content.perform(action),
                M::OpenAiSimilaritySearchContent(action) => self.openai_doc_search.perform(action),
                M::MistralSimilaritySearchContent(action) => {
                    self.mistral_doc_search.perform(action)
                }
                M::FtsContent(action) => self.fts_text.perform(action),
                M::SeeLinksContent(action) => self.seelinks_text.perform(action),
            }
        });

        self.rt = Some(rt);
    }

    async fn link_documents(&self, documents: String) -> String {
        let documents = documents.trim();
        let Some((one, two)) = documents.split_once(",") else {
            return "Please insert two document names separated by a comma".to_string();
        };
        let one = RecordId::from_table_key("document", one);
        let two = RecordId::from_table_key("document", two);

        match self
            .db
            .query("RELATE $one->link->$two;")
            .bind(("one", one))
            .bind(("two", two))
            .await
        {
            Ok(mut r) => match r.take::<Value>(0) {
                Ok(val) => format!("Link added: {val}"),
                Err(e) => e.to_string(),
            },
            Err(e) => e.to_string(),
        }
    }

    async fn raw_query(&self, query: &str) -> String {
        match self.db.query(query).await {
            Ok(mut r) => {
                let mut results = vec![];
                let num_statements = r.num_statements();
                for index in 0..num_statements {
                    match r.take::<Value>(index) {
                        Ok(good) => results.push(good.to_string()),
                        Err(e) => results.push(e.to_string()),
                    }
                }
                results.join("\n")
            }
            Err(e) => e.to_string(),
        }
    }

    async fn see_docs(&self) -> String {
        let res = self
            .raw_query("(SELECT VALUE title FROM document).sort()")
            .await;
        format!("All database article titles: {res}")
    }

    async fn fts_search(&self, input: String) -> Result<String, String> {
        let input = input.trim().to_owned();
        match self
            .db
            .query(
                "SELECT 
                search::highlight('**', '**', 0) AS title, 
                search::highlight('**', '**', 1) AS extract,
(search::score(0) * 3) + search::score(1) AS score
                FROM document
            WHERE title @0@ $input OR extract @1@ $input
            ORDER BY score DESC;",
            )
            .bind(("input", input))
            .await
        {
            Ok(mut res) => Ok(res.take::<Value>(0).map_err(|e| e.to_string())?.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn linked_docs(&self, doc: String) -> String {
        let doc = doc.trim();
        self.raw_query(&format!(
            "type::thing('document', '{doc}').{{..3}}.{{ id, next: ->link->document.@ }};
                "
        ))
        .await
    }

    fn view(&self) -> Element<Message> {
        column![
            row![iced::widget::image("surreal.png").height(100).width(200)],
            row![
                button("Insert document")
                    .width(190)
                    .on_press(Message::InsertDocuments),
                text_editor(&self.document_content)
                    .placeholder("Add Wikipedia article title to insert, separated by commas")
                    .on_action(Message::InsertDocumentsContent),
            ],
            row![text("")],
            row![
                button("Link documents")
                    .width(190)
                    .on_press(Message::LinkDocuments),
                text_editor(&self.link_content)
                    .placeholder("Enter two document titles, separated by a comma")
                    .on_action(Message::LinkDocumentsContent),
            ],
            row![
                button("Link unlinked docs")
                    .width(190)
                    .on_press(Message::TryLink),
                text_input(
                    "Try to create links for docs based on possible article names in their summary",
                    ""
                )
            ],
            row![text("")],
            row![
                button("Add OpenAI embeddings")
                    .width(190)
                    .on_press(Message::AddOpenAi),
                text_input(
                    "Add OpenAI embeddings to documents that do not have them",
                    ""
                )
            ],
            row![
                button("Add Mistral embeddings")
                    .width(190)
                    .on_press(Message::AddMistral),
                text_input(
                    "Add Mistral embeddings to documents that do not have them",
                    ""
                )
            ],
            row![
                button("OpenAI similarity search")
                    .width(190)
                    .on_press(Message::OpenAiSimilaritySearch),
                text_editor(&self.openai_doc_search)
                    .placeholder("Enter document name")
                    .on_action(Message::OpenAiSimilaritySearchContent),
            ],
            row![
                button("Mistral similarity search")
                    .width(190)
                    .on_press(Message::MistralSimilaritySearch),
                text_editor(&self.mistral_doc_search)
                    .placeholder("Enter document name")
                    .on_action(Message::MistralSimilaritySearchContent),
            ],
            row![
                button("Full text search").width(190).on_press(Message::Fts),
                text_editor(&self.fts_text)
                    .placeholder("Find a document via full-text search")
                    .on_action(Message::FtsContent),
            ],
            row![
                button("See linked articles")
                    .width(190)
                    .on_press(Message::SeeLinks),
                text_editor(&self.seelinks_text)
                    .placeholder("Finds all linked aticles down to a depth of 3")
                    .on_action(Message::SeeLinksContent),
            ],
            row![text("")],
            row![
                button("See all document titles")
                    .width(190)
                    .on_press(Message::SeeDocs),
                text_input("See titles of all documents in the database", "")
            ],
            row![text("")],
            row![
                button("Run query").width(190).on_press(Message::Query),
                text_editor(&self.query_content)
                    .placeholder("Run any raw SurrealQL query")
                    .on_action(Message::QueryContent),
            ],
            scrollable(text(&self.app_output).size(19).center()).width(1000)
        ]
        .padding(50)
        .align_x(Center)
        .into()
    }
}

fn main() -> Result<(), iced::Error> {
    let surreal = Palette {
        background: color!(0x15131D),
        text: color!(0xF9F9F9),
        primary: color!(0x242133),
        success: color!(0xFF00A0),
        danger: color!(0xFF00A0),
    };

    let custom = Arc::new(Custom::new("Surreal".to_string(), surreal));
    let cloned = Theme::Custom(custom);
    iced::application(
        "SurrealDB AI-native multi-model demo UI",
        App::update,
        App::view,
    )
    .theme(move |_| cloned.clone())
    .run()?;
    Ok(())
}