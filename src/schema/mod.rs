//! Schema module for GCP Workflows definitions

mod workflows;

pub use workflows::{
    is_step_action, is_workflow_keyword, step_action_set, workflow_keyword_set, CALL_STEP_KEYWORDS,
    FOR_STEP_KEYWORDS, PARALLEL_STEP_KEYWORDS, RETRY_KEYWORDS, STEP_ACTION_KEYWORDS,
    SUBWORKFLOW_KEYWORDS, SWITCH_CONDITION_KEYWORDS, SWITCH_STEP_KEYWORDS, TRY_STEP_KEYWORDS,
    WORKFLOW_KEYWORDS,
};
