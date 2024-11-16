use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use std::collections::HashMap;

use super::{Image, Link, LinkId};

#[derive(Default, Debug, Serialize)]
pub struct LinkGraph {
    links: HashMap<LinkId, Link>,
    link_ids: HashMap<String, LinkId>,
}

impl LinkGraph {
    pub fn update(
        &mut self,
        url: &str,
        parent: &str,
        children: &[String],
        images: &[Image],
        titles: &[String],
    ) -> Result<()> {
        let maybe_parent = self.link_ids.get(parent).cloned();

        let valid_children: Vec<LinkId> = children
            .iter()
            .filter_map(|c| self.link_ids.get(c).cloned())
            .collect();

        let link = self.force_get_link_id(url)?;

        if let Some(parent_id) = maybe_parent {
            link.parents.push(parent_id);
        }

        link.children.extend(valid_children);
        link.images.extend(images.iter().cloned());
        link.titles.extend(titles.iter().cloned());
        let this_link_id = link.id;

        if let Some(parent_id) = maybe_parent {

            let parent_link = self
                .links
                .get_mut(&parent_id)
                .context("could not find parent link")?;

            parent_link.children.push(this_link_id);
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.links.len()
    }

    pub fn link_visited(&self, url: &str) -> bool {
        self.link_ids.get(url).is_some()
    }

    fn force_get_link_id(&mut self, url: &str) -> Result<&mut Link> {
        let this_link_id = if let Some(link_id) = self.link_ids.get(url) {
            *link_id
        } else {
            let new_link = Link {
                url: url.to_string(),
                ..Default::default()
            };
            let new_link_id = new_link.id;

            self.links
                .insert(new_link_id, new_link)
                .map_or(Ok(()), |_| Err(anyhow!("link already exists")))?;

            new_link_id
        };

        self.link_ids.insert(url.to_string(), this_link_id);
        self.links
            .get_mut(&this_link_id)
            .ok_or_else(|| anyhow!("failed to get link"))
    }
}

impl<'a> IntoIterator for &'a LinkGraph {
    type Item = (&'a LinkId, &'a Link);
    type IntoIter = std::collections::hash_map::Iter<'a, LinkId, Link>;

    fn into_iter(self) -> Self::IntoIter {
        self.links.iter()
    }
}
//Author: Morteza Farrokhnejad