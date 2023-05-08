use sctk::{
    data_device_manager::{data_offer::DragOffer, ReadPipe},
    reexports::client::protocol::wl_data_device_manager::DndAction,
};
use std::{
    os::fd::{AsRawFd, OwnedFd, RawFd},
    sync::{Arc, Mutex},
};

/// Dnd Offer events
#[derive(Debug, Clone, PartialEq)]
pub enum DndOfferEvent {
    /// A DnD offer has been introduced with the given mime types.
    Enter {
        /// x coordinate of the offer
        x: f64,
        /// y coordinate of the offer
        y: f64,
        /// The offered mime types
        mime_types: Vec<String>,
    },
    /// The DnD device has left.
    Leave,
    /// Drag and Drop Motion event.
    Motion {
        /// x coordinate of the pointer
        x: f64,
        /// y coordinate of the pointer
        y: f64,
    },
    /// The selected DnD action
    SelectedAction(DndAction),
    /// The offered actions for the current DnD offer
    SourceActions(DndAction),
    /// Dnd Drop event
    DropPerformed,
    /// Raw DnD Data
    DndData {
        /// The data
        data: Vec<u8>,
        /// The mime type of the data
        mime_type: String,
    },
    /// Raw Selection Data
    SelectionData {
        /// The data
        data: Vec<u8>,
        /// The mime type of the data
        mime_type: String,
    },
    /// Selection Offer
    /// a selection offer has been introduced with the given mime types.
    SelectionOffer(Vec<String>),
}

/// Selection Offer events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionOfferEvent {
    /// a selection offer has been introduced with the given mime types.
    Offer(Vec<String>),
    /// Read the Selection data
    Data {
        /// The mime type that the selection should be converted to.
        mime_type: String,
        /// The data
        data: Vec<u8>,
    },
}

/// A ReadPipe and the mime type of the data.
#[derive(Debug, Clone)]
pub struct ReadData {
    /// mime type of the data
    pub mime_type: String,
    /// The pipe to read the data from
    pub fd: Arc<Mutex<ReadPipe>>,
}

impl ReadData {
    /// Create a new ReadData
    pub fn new(mime_type: String, fd: Arc<Mutex<ReadPipe>>) -> Self {
        Self { mime_type, fd }
    }
}

/// Data Source events
/// Includes drag and drop events and clipboard events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataSourceEvent {
    /// A Dnd action was selected by the compositor for your source.
    DndActionAccepted(DndAction),
    /// A mime type was accepted by a client for your source.
    MimeAccepted(Option<String>),
    /// Some client has requested the DnD data.
    /// This is used to send the data to the client.
    SendDndData(String),
    /// Some client has requested the selection data.
    /// This is used to send the data to the client.
    SendSelectionData(String),
    /// The data source has been cancelled and is no longer valid.
    /// This may be sent for multiple reasons
    Cancelled,
    /// Dnd Finished
    DndFinished,
    /// Dnd Drop event
    DndDropPerformed,
}

/// A WriteData and the mime type of the data to be written.
#[derive(Debug, Clone)]
pub struct WriteData {
    /// mime type of the data
    pub mime_type: String,
    /// The fd to write the data to
    pub fd: Arc<Mutex<OwnedFd>>,
}

impl WriteData {
    /// Create a new WriteData
    pub fn new(mime_type: String, fd: Arc<Mutex<OwnedFd>>) -> Self {
        Self { mime_type, fd }
    }
}

impl PartialEq for WriteData {
    fn eq(&self, other: &Self) -> bool {
        self.fd.lock().unwrap().as_raw_fd()
            == other.fd.lock().unwrap().as_raw_fd()
    }
}

impl Eq for WriteData {}

impl PartialEq for ReadData {
    fn eq(&self, other: &Self) -> bool {
        self.fd.lock().unwrap().as_raw_fd()
            == other.fd.lock().unwrap().as_raw_fd()
    }
}

impl Eq for ReadData {}
