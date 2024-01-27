import requests
import os

def get_merged_prs(repo, milestone, token):
    url = f'https://api.github.com/repos/{repo}/pulls'
    params = {
        'state': 'closed',
        'per_page': 100,  # Number of items per page, adjust as needed
    }
    headers = {'Authorization': f'token {token}'}

    all_prs = []
    page = 1

    while True:
        params['page'] = page
        response = requests.get(url, params=params, headers=headers)
        response.raise_for_status()

        prs = response.json()

        if not prs:
            break  # No more pages

        all_prs.extend([pr for pr in prs if pr['merged_at'] and (pr['milestone'] or {}).get('title', '') == milestone])
        page += 1

    return all_prs

def categorize_prs(prs):
    categorized_prs = {'addition': [], 'change': [], 'fix': []}

    for pr in prs:
        labels = [label['name'] for label in pr['labels']]
        if 'addition' in labels or 'feature' in labels:
            categorized_prs['addition'].append(pr)
        elif 'fix' in labels or 'bug' in labels:
            categorized_prs['fix'].append(pr)
        elif 'change' in labels or 'improvement' in labels:
            categorized_prs['change'].append(pr)

    return categorized_prs

def get_authors(prs):
    authors = set()
    for pr in prs:
        authors.add(pr['user']['login'])
    return sorted(authors, key=str.casefold)

def main():
    repo = 'iced-rs/iced'
    milestone = '0.12'
    token = os.environ['GITHUB_TOKEN']

    prs = get_merged_prs(repo, milestone, token)
    categorized_prs = categorize_prs(prs)

    for category, items in categorized_prs.items():
        print(f"### {category.capitalize()}")

        for pr in items:
            print(f"- {pr['title']}. [#{pr['number']}](https://github.com/{repo}/pull/{pr['number']})")

        print("")

    print("")

    authors = get_authors(prs)

    print("Many thanks to...")
    for author in authors:
        print(f"- @{author}")

if __name__ == "__main__":
    main()