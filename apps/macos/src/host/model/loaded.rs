//! 后台加载好的本地模型。

use std::sync::Arc;

use kasm_core::ChoiceModel;
use kasm_core::sentence::SentenceScorer;

/// 加载线程送回来的模型：整句重打分（含章）或选词重排（知意，试验）。
pub enum LoadedModel {
    Sentence(Box<dyn SentenceScorer>),
    Choice(Arc<dyn ChoiceModel>),
}
