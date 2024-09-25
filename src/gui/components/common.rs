pub trait Component {
    fn ui(&mut self, ctx: &egui::Context);
}

pub enum InterfaceAction {
    Draw,
    None,
}
