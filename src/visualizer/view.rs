use crate::visualizer::widgets::Widget;

pub(crate) trait View {
    type Result: Widget<Self>;
    fn to_widget(&self) -> Self::Result;
}
