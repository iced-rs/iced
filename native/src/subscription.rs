//! Listen to external events in your application.
use crate::core::event::{self, Event};
use crate::core::window;
use crate::core::Hasher;
use crate::futures::futures::{self, Future, Stream};
use crate::futures::subscription::{EventStream, Recipe, Subscription};
use crate::futures::{BoxStream, MaybeSend};

use std::hash::Hash;
