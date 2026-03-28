use std::rc::Rc;

use dominator::{Dom, html};

use super::items::item_list;
use super::state::AppState;

pub fn app_root(state: Rc<AppState>) -> Dom {
    html!("div", {
        .style("min-height", "100vh")
        .style("color", "white")
        .style("padding", "2rem")
        .children(&mut [
            html!("h1", {
                .style("font-size", "1.875rem")
                .style("font-weight", "bold")
                .style("margin-bottom", "2rem")
                .text("Inventory")
            }),
            item_list(state),
        ])
    })
}
