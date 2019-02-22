use imgui::*;

pub fn use_darcula(style: &mut ImGuiStyle) {
    style.colors[ImGuiCol::Text as usize as usize] =
        ImVec4::new(0.73333335, 0.73333335, 0.73333335, 1.00);
    style.colors[ImGuiCol::TextDisabled as usize] =
        ImVec4::new(0.34509805, 0.34509805, 0.34509805, 1.00);
    style.colors[ImGuiCol::WindowBg as usize] =
        ImVec4::new(0.23529413, 0.24705884, 0.25490198, 0.94);
    style.colors[ImGuiCol::ChildBg as usize] =
        ImVec4::new(0.23529413, 0.24705884, 0.25490198, 0.00);
    style.colors[ImGuiCol::PopupBg as usize] =
        ImVec4::new(0.23529413, 0.24705884, 0.25490198, 0.94);
    style.colors[ImGuiCol::Border as usize] = ImVec4::new(0.33333334, 0.33333334, 0.33333334, 0.50);
    style.colors[ImGuiCol::BorderShadow as usize] =
        ImVec4::new(0.15686275, 0.15686275, 0.15686275, 0.00);
    style.colors[ImGuiCol::FrameBg as usize] =
        ImVec4::new(0.16862746, 0.16862746, 0.16862746, 0.54);
    style.colors[ImGuiCol::FrameBgHovered as usize] =
        ImVec4::new(0.453125, 0.67578125, 0.99609375, 0.67);
    style.colors[ImGuiCol::FrameBgActive as usize] =
        ImVec4::new(0.47058827, 0.47058827, 0.47058827, 0.67);
    style.colors[ImGuiCol::TitleBg as usize] = ImVec4::new(0.04, 0.04, 0.04, 1.00);
    style.colors[ImGuiCol::TitleBgCollapsed as usize] = ImVec4::new(0.16, 0.29, 0.48, 1.00);
    style.colors[ImGuiCol::TitleBgActive as usize] = ImVec4::new(0.00, 0.00, 0.00, 0.51);
    style.colors[ImGuiCol::MenuBarBg as usize] =
        ImVec4::new(0.27058825, 0.28627452, 0.2901961, 0.80);
    style.colors[ImGuiCol::ScrollbarBg as usize] =
        ImVec4::new(0.27058825, 0.28627452, 0.2901961, 0.60);
    style.colors[ImGuiCol::ScrollbarGrab as usize] =
        ImVec4::new(0.21960786, 0.30980393, 0.41960788, 0.51);
    style.colors[ImGuiCol::ScrollbarGrabHovered as usize] =
        ImVec4::new(0.21960786, 0.30980393, 0.41960788, 1.00);
    style.colors[ImGuiCol::ScrollbarGrabActive as usize] =
        ImVec4::new(0.13725491, 0.19215688, 0.2627451, 0.91);
    // style.colors[ImGuiCol::ComboBg]               = ImVec4::new(0.1, 0.1, 0.1, 0.99);
    style.colors[ImGuiCol::CheckMark as usize] = ImVec4::new(0.90, 0.90, 0.90, 0.83);
    style.colors[ImGuiCol::SliderGrab as usize] = ImVec4::new(0.70, 0.70, 0.70, 0.62);
    style.colors[ImGuiCol::SliderGrabActive as usize] = ImVec4::new(0.30, 0.30, 0.30, 0.84);
    style.colors[ImGuiCol::Button as usize] = ImVec4::new(0.33333334, 0.3529412, 0.36078432, 0.49);
    style.colors[ImGuiCol::ButtonHovered as usize] =
        ImVec4::new(0.21960786, 0.30980393, 0.41960788, 1.00);
    style.colors[ImGuiCol::ButtonActive as usize] =
        ImVec4::new(0.13725491, 0.19215688, 0.2627451, 1.00);
    style.colors[ImGuiCol::Header as usize] = ImVec4::new(0.33333334, 0.3529412, 0.36078432, 0.53);
    style.colors[ImGuiCol::HeaderHovered as usize] =
        ImVec4::new(0.453125, 0.67578125, 0.99609375, 0.67);
    style.colors[ImGuiCol::HeaderActive as usize] =
        ImVec4::new(0.47058827, 0.47058827, 0.47058827, 0.67);
    style.colors[ImGuiCol::Separator as usize] =
        ImVec4::new(0.31640625, 0.31640625, 0.31640625, 1.00);
    style.colors[ImGuiCol::SeparatorHovered as usize] =
        ImVec4::new(0.31640625, 0.31640625, 0.31640625, 1.00);
    style.colors[ImGuiCol::SeparatorActive as usize] =
        ImVec4::new(0.31640625, 0.31640625, 0.31640625, 1.00);
    style.colors[ImGuiCol::ResizeGrip as usize] = ImVec4::new(1.00, 1.00, 1.00, 0.85);
    style.colors[ImGuiCol::ResizeGripHovered as usize] = ImVec4::new(1.00, 1.00, 1.00, 0.60);
    style.colors[ImGuiCol::ResizeGripActive as usize] = ImVec4::new(1.00, 1.00, 1.00, 0.90);
    style.colors[ImGuiCol::PlotLines as usize] = ImVec4::new(0.61, 0.61, 0.61, 1.00);
    style.colors[ImGuiCol::PlotLinesHovered as usize] = ImVec4::new(1.00, 0.43, 0.35, 1.00);
    style.colors[ImGuiCol::PlotHistogram as usize] = ImVec4::new(0.90, 0.70, 0.00, 1.00);
    style.colors[ImGuiCol::PlotHistogramHovered as usize] = ImVec4::new(1.00, 0.60, 0.00, 1.00);
    style.colors[ImGuiCol::TextSelectedBg as usize] =
        ImVec4::new(0.18431373, 0.39607847, 0.79215693, 0.90);
}

pub fn use_light(style: &mut ImGuiStyle) {
    style.colors[ImGuiCol::Text as usize] = ImVec4::new(0.00, 0.00, 0.00, 1.00);
    style.colors[ImGuiCol::TextDisabled as usize] = ImVec4::new(0.60, 0.60, 0.60, 1.00);
    style.colors[ImGuiCol::WindowBg as usize] = ImVec4::new(0.94, 0.94, 0.94, 0.94);
    style.colors[ImGuiCol::PopupBg as usize] = ImVec4::new(1.00, 1.00, 1.00, 0.94);
    style.colors[ImGuiCol::Border as usize] = ImVec4::new(0.00, 0.00, 0.00, 0.39);
    style.colors[ImGuiCol::BorderShadow as usize] = ImVec4::new(1.00, 1.00, 1.00, 0.10);
    style.colors[ImGuiCol::FrameBg as usize] = ImVec4::new(1.00, 1.00, 1.00, 0.94);
    style.colors[ImGuiCol::FrameBgHovered as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.40);
    style.colors[ImGuiCol::FrameBgActive as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.67);
    style.colors[ImGuiCol::TitleBg as usize] = ImVec4::new(0.96, 0.96, 0.96, 1.00);
    style.colors[ImGuiCol::TitleBgCollapsed as usize] = ImVec4::new(1.00, 1.00, 1.00, 0.51);
    style.colors[ImGuiCol::TitleBgActive as usize] = ImVec4::new(0.82, 0.82, 0.82, 1.00);
    style.colors[ImGuiCol::MenuBarBg as usize] = ImVec4::new(0.86, 0.86, 0.86, 1.00);
    style.colors[ImGuiCol::ScrollbarBg as usize] = ImVec4::new(0.98, 0.98, 0.98, 0.53);
    style.colors[ImGuiCol::ScrollbarGrab as usize] = ImVec4::new(0.69, 0.69, 0.69, 1.00);
    style.colors[ImGuiCol::ScrollbarGrabHovered as usize] = ImVec4::new(0.59, 0.59, 0.59, 1.00);
    style.colors[ImGuiCol::ScrollbarGrabActive as usize] = ImVec4::new(0.49, 0.49, 0.49, 1.00);
    style.colors[ImGuiCol::CheckMark as usize] = ImVec4::new(0.26, 0.59, 0.98, 1.00);
    style.colors[ImGuiCol::SliderGrab as usize] = ImVec4::new(0.24, 0.52, 0.88, 1.00);
    style.colors[ImGuiCol::SliderGrabActive as usize] = ImVec4::new(0.26, 0.59, 0.98, 1.00);
    style.colors[ImGuiCol::Button as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.40);
    style.colors[ImGuiCol::ButtonHovered as usize] = ImVec4::new(0.26, 0.59, 0.98, 1.00);
    style.colors[ImGuiCol::ButtonActive as usize] = ImVec4::new(0.06, 0.53, 0.98, 1.00);
    style.colors[ImGuiCol::Header as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.31);
    style.colors[ImGuiCol::HeaderHovered as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.80);
    style.colors[ImGuiCol::HeaderActive as usize] = ImVec4::new(0.26, 0.59, 0.98, 1.00);
    style.colors[ImGuiCol::ResizeGrip as usize] = ImVec4::new(1.00, 1.00, 1.00, 0.50);
    style.colors[ImGuiCol::ResizeGripHovered as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.67);
    style.colors[ImGuiCol::ResizeGripActive as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.95);
    style.colors[ImGuiCol::PlotLines as usize] = ImVec4::new(0.39, 0.39, 0.39, 1.00);
    style.colors[ImGuiCol::PlotLinesHovered as usize] = ImVec4::new(1.00, 0.43, 0.35, 1.00);
    style.colors[ImGuiCol::PlotHistogram as usize] = ImVec4::new(0.90, 0.70, 0.00, 1.00);
    style.colors[ImGuiCol::PlotHistogramHovered as usize] = ImVec4::new(1.00, 0.60, 0.00, 1.00);
    style.colors[ImGuiCol::TextSelectedBg as usize] = ImVec4::new(0.26, 0.59, 0.98, 0.35);
}
