use imgui::*;

pub struct TabBar {
    tab_text: Vec<String>,
    curr_tab: usize,
}

impl TabBar {
    pub fn new(tab_text: Vec<String>) -> TabBar {
        TabBar {
            tab_text,
            curr_tab: 0,
        }
    }

    pub fn curr_tab(&self) -> usize {
        self.curr_tab
    }

    pub fn build<'ui>(&mut self, ui: &'ui Ui) -> usize {
        let start_pos = ui.get_cursor_screen_pos();

        for (i, tab_text) in self.tab_text.iter().enumerate() {
            if i == self.curr_tab {
                let style = ui.imgui().style();
                let active_tab_colors = [
                    (
                        ImGuiCol::Button,
                        style.colors[ImGuiCol::ButtonActive as usize],
                    ),
                    (
                        ImGuiCol::ButtonHovered,
                        style.colors[ImGuiCol::ButtonActive as usize],
                    ),
                ];
                ui.with_color_vars(&active_tab_colors, || {
                    TabBar::tab_button(ui, im_str!("{}", tab_text));
                })
            } else {
                if TabBar::tab_button(ui, im_str!("{}", tab_text)) {
                    self.curr_tab = i;
                }
            }
        }

        // move cursor to below tab bar
        let label_size = ui.calc_text_size(im_str!("f"), true, 100.0);
        let style = ui.imgui().style();
        let new_pos = (
            start_pos.0,
            start_pos.1 + label_size.y + 6.0 * style.frame_padding.y,
        );
        ui.set_cursor_screen_pos(new_pos);

        self.curr_tab
    }

    fn tab_button<'ui>(ui: &'ui Ui, label: &ImStr) -> bool {
        let style = ui.imgui().style();
        let label_size = ui.calc_text_size(label, true, 100.0);

        let button_pos = ui.get_cursor_screen_pos();
        let button_size = (
            label_size.x + 4.0 * style.frame_padding.x,
            label_size.y + 4.0 * style.frame_padding.y,
        );

        let label_pos = (
            button_pos.0 + 2.0 * style.frame_padding.x,
            button_pos.1 + 2.0 * style.frame_padding.y,
        );

        // check if active, hovered, clicked, etc.
        let mut button_color: ImVec4 = style.colors[ImGuiCol::Button as usize];
        let mut button_clicked = false;
        let mouse_pos = ui.imgui().mouse_pos();
        if mouse_pos.0 >= button_pos.0
            && mouse_pos.0 <= button_pos.0 + button_size.0
            && mouse_pos.1 >= button_pos.1
            && mouse_pos.1 <= button_pos.1 + button_size.1
        {
            if ui.imgui().is_mouse_clicked(ImMouseButton::Left) {
                button_color = style.colors[ImGuiCol::ButtonActive as usize];
                button_clicked = true;
            } else {
                button_color = style.colors[ImGuiCol::ButtonHovered as usize];
            }
        }

        let draw_list = ui.get_window_draw_list();
        draw_list
            .add_rect(
                button_pos,
                (button_pos.0 + button_size.0, button_pos.1 + button_size.1),
                button_color,
            )
            .rounding(10.0)
            .round_top_left(true)
            .round_top_right(true)
            .round_bot_left(false)
            .round_bot_right(false)
            .filled(true)
            .build();

        draw_list.add_text(label_pos, style.colors[ImGuiCol::Text as usize], label);

        ui.set_cursor_screen_pos((button_pos.0 + button_size.0 + 1.0, button_pos.1));

        button_clicked
    }
}
