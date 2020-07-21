use crate::instance::tag::{Resource, TagRepo};
use crate::{CLAIRE, INVESTIGATION_TAG_KEY};

pub struct ClearInvestigationService {
    tag_repo: TagRepo,
}

impl ClearInvestigationService {
    pub fn new() -> ClearInvestigationService {
        ClearInvestigationService {
            tag_repo: TagRepo::new(),
        }
    }

    pub async fn clear_investigation(
        &self,
        investigation_id: &str,
    ) -> Result<Vec<Resource>, Box<dyn std::error::Error>> {
        let resources = self
            .tag_repo
            .get_resources(INVESTIGATION_TAG_KEY, investigation_id)
            .await?
            .iter()
            .filter(|r| !r.is_snapshot())
            .cloned()
            .collect::<Vec<Resource>>();

        self.untag_resources(&resources).await?;

        Ok(resources)
    }

    pub async fn untag_resources(
        &self,
        resources: &Vec<Resource>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if resources.len() == 0 {
            return Ok(());
        }

        self.tag_repo
            .delete_tags(resources, &vec![CLAIRE, INVESTIGATION_TAG_KEY])
            .await?;

        Ok(())
    }
}
