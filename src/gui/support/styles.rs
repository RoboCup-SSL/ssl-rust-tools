use imgui::*;

pub fn use_dark(style: &mut ImGuiStyle) {
    style.colors[ImGuiCol::Text as usize] = ImVec4::new(1.00, 1.00, 1.00, 1.00);
    style.colors[ImGuiCol::TextDisabled as usize] = ImVec4::new(0.50, 0.50, 0.50, 1.00);
    style.colors[ImGuiCol::WindowBg as usize] = ImVec4::new(0.06, 0.06, 0.06, 0.98);
    style.colors[ImGuiCol::ChildBg as usize] = ImVec4::new(1.00, 1.00, 1.00, 0.00);
    style.colors[ImGuiCol::PopupBg as usize] = ImVec4::new(0.08, 0.08, 0.08, 0.94);
    style.colors[ImGuiCol::Border as usize] = ImVec4::new(0.43, 0.43, 0.50, 0.50);
    style.colors[ImGuiCol::BorderShadow as usize] = ImVec4::new(0.00, 0.00, 0.00, 0.00);
    style.colors[ImGuiCol::FrameBg as usize] = ImVec4::new(0.16, 0.29, 0.48, 0.54);
    style.colors[ImGuiCol::FrameBgHovered as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.40);
    style.colors[ImGuiCol::FrameBgActive as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.67);
    style.colors[ImGuiCol::TitleBg as usize] = ImVec4::new(0.04, 0.04, 0.04, 1.00);
    style.colors[ImGuiCol::TitleBgActive as usize] = ImVec4::new(0.16, 0.29, 0.48, 1.00);
    style.colors[ImGuiCol::TitleBgCollapsed as usize] = ImVec4::new(0.00, 0.00, 0.00, 0.51);
    style.colors[ImGuiCol::MenuBarBg as usize] = ImVec4::new(0.14, 0.14, 0.14, 1.00);
    style.colors[ImGuiCol::ScrollbarBg as usize] = ImVec4::new(0.02, 0.02, 0.02, 0.53);
    style.colors[ImGuiCol::ScrollbarGrab as usize] = ImVec4::new(0.31, 0.31, 0.31, 1.00);
    style.colors[ImGuiCol::ScrollbarGrabHovered as usize] = ImVec4::new(0.41, 0.41, 0.41, 1.00);
    style.colors[ImGuiCol::ScrollbarGrabActive as usize] = ImVec4::new(0.51, 0.51, 0.51, 1.00);
    style.colors[ImGuiCol::CheckMark as usize] = ImVec4::new(0.26, 0.59, 0.98, 1.00);
    style.colors[ImGuiCol::SliderGrab as usize] = ImVec4::new(0.24, 0.52, 0.88, 1.00);
    style.colors[ImGuiCol::SliderGrabActive as usize] = ImVec4::new(0.26, 0.59, 0.98, 1.00);
    style.colors[ImGuiCol::Button as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.40);
    style.colors[ImGuiCol::ButtonHovered as usize] = ImVec4::new(0.26, 0.59, 0.98, 1.00);
    style.colors[ImGuiCol::ButtonActive as usize] = ImVec4::new(0.06, 0.53, 0.98, 1.00);
    style.colors[ImGuiCol::Header as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.31);
    style.colors[ImGuiCol::HeaderHovered as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.80);
    style.colors[ImGuiCol::HeaderActive as usize] = ImVec4::new(0.26, 0.59, 0.98, 1.00);
    style.colors[ImGuiCol::Separator as usize] = ImVec4::new(0.43, 0.43, 0.50, 0.50);
    style.colors[ImGuiCol::SeparatorHovered as usize] = ImVec4::new(0.10, 0.40, 0.75, 0.78);
    style.colors[ImGuiCol::SeparatorActive as usize] = ImVec4::new(0.10, 0.40, 0.75, 1.00);
    style.colors[ImGuiCol::ResizeGrip as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.25);
    style.colors[ImGuiCol::ResizeGripHovered as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.67);
    style.colors[ImGuiCol::ResizeGripActive as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.95);
    style.colors[ImGuiCol::PlotLines as usize] = ImVec4::new(0.61, 0.61, 0.61, 1.00);
    style.colors[ImGuiCol::PlotLinesHovered as usize] = ImVec4::new(1.00, 0.43, 0.35, 1.00);
    style.colors[ImGuiCol::PlotHistogram as usize] = ImVec4::new(0.90, 0.70, 0.00, 1.00);
    style.colors[ImGuiCol::PlotHistogramHovered as usize] = ImVec4::new(1.00, 0.60, 0.00, 1.00);
    style.colors[ImGuiCol::TextSelectedBg as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.35);
    style.colors[ImGuiCol::DragDropTarget as usize] = ImVec4::new(1.00, 1.00, 0.00, 0.90);
    style.colors[ImGuiCol::NavHighlight as usize] = ImVec4::new(0.26, 0.59, 0.98, 1.00);
    style.colors[ImGuiCol::NavWindowingHighlight as usize] = ImVec4::new(1.00, 1.00, 1.00, 0.70);
    style.colors[ImGuiCol::NavWindowingDimBg as usize] = ImVec4::new(0.80, 0.80, 0.80, 0.20);
    style.colors[ImGuiCol::ModalWindowDimBg as usize] = ImVec4::new(0.80, 0.80, 0.80, 0.35);
}
