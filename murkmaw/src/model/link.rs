use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::model::image::Image;

static LINK_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Type for the Link ID
pub type LinkId = u64;

#[derive(Debug, Serialize)]
pub struct Link {
    /// unique ID for this link
    pub id: LinkId,
    /// the URL string for this link
    pub url: String,
    /// list of links contained inside this webpage
    pub children: Vec<LinkId>,
    /// list of webages that link to this webpage
    pub parents: Vec<LinkId>,
    /// list of images found on the webpage
    pub images: Vec<Image>,
    /// list of titles found on this webpage
    pub titles: Vec<String>,
}

impl Default for Link {
    fn default() -> Link {
        Link {
            id: LINK_ID_COUNTER.fetch_add(1, Ordering::SeqCst),
            url: String::from(""),
            children: Default::default(),
            parents: Default::default(),
            images: Default::default(),
            titles: Default::default(),
        }
    }
}
//Author: Morteza Farrokhnejad

impl Link {
    pub fn new(
        url: String,
        children: Vec<LinkId>,
        parents: Vec<LinkId>,
        images: Vec<Image>,
        titles: Vec<String>,
    ) -> Link {
        let id = LINK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        Link {
            url,
            id,
            children,
            parents,
            images,
            titles,
        }
    }
}