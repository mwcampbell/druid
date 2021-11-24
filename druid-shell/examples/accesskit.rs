// Copyright 2018 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{any::Any, num::NonZeroU64};

use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};

use druid_shell::kurbo::Size;
use druid_shell::piet::{Color, RenderContext};

use druid_shell::{Application, KbKey, KeyEvent, Region, WinHandler, WindowBuilder, WindowHandle};

const BG_COLOR: Color = Color::rgb8(0x27, 0x28, 0x22);

const WINDOW_TITLE: &str = "Hello world";

const WINDOW_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(1) });
const BUTTON_1_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(2) });
const BUTTON_2_ID: NodeId = NodeId(unsafe { NonZeroU64::new_unchecked(3) });
const INITIAL_FOCUS: NodeId = BUTTON_1_ID;

fn make_button(id: NodeId, name: &str) -> Node {
    Node {
        name: Some(name.into()),
        focusable: true,
        ..Node::new(id, Role::Button)
    }
}

fn get_initial_state() -> TreeUpdate {
    let root = Node {
        children: Box::new([BUTTON_1_ID, BUTTON_2_ID]),
        name: Some(WINDOW_TITLE.into()),
        ..Node::new(WINDOW_ID, Role::Window)
    };
    let button_1 = make_button(BUTTON_1_ID, "Button 1");
    let button_2 = make_button(BUTTON_2_ID, "Button 2");
    TreeUpdate {
        clear: None,
        nodes: vec![root, button_1, button_2],
        tree: Some(Tree::new(
            TreeId("test".into()),
            WINDOW_ID,
            StringEncoding::Utf8,
        )),
        focus: None,
    }
}

struct HelloState {
    size: Size,
    handle: WindowHandle,
    focus: NodeId,
}

impl HelloState {
    fn update_focus(&self, is_window_focused: bool) {
        let update = TreeUpdate {
            clear: None,
            nodes: vec![],
            tree: None,
            focus: is_window_focused.then(|| self.focus),
        };
        self.handle.update_accesskit(update);
    }
}

impl Default for HelloState {
    fn default() -> Self {
        Self {
            size: Default::default(),
            handle: Default::default(),
            focus: INITIAL_FOCUS,
        }
    }
}

impl WinHandler for HelloState {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
        self.handle.init_accesskit(get_initial_state());
    }

    fn prepare_paint(&mut self) {}

    fn paint(&mut self, piet: &mut piet_common::Piet, _: &Region) {
        let rect = self.size.to_rect();
        piet.fill(rect, &BG_COLOR);
    }

    fn key_down(&mut self, event: KeyEvent) -> bool {
        if event.key == KbKey::Tab {
            self.focus = if self.focus == BUTTON_1_ID {
                BUTTON_2_ID
            } else {
                BUTTON_1_ID
            };
            self.update_focus(true);
            return true;
        }
        if event.key == KbKey::Enter || event.key == KbKey::Character(" ".into()) {
            // This is a pretty hacky way of updating a node.
            // A real GUI framework would have a consistent way
            // of building a node from underlying data.
            let node = if self.focus == BUTTON_1_ID {
                make_button(BUTTON_1_ID, "You pressed button 1")
            } else {
                make_button(BUTTON_2_ID, "You pressed button 2")
            };
            let update = TreeUpdate {
                clear: None,
                nodes: vec![node],
                tree: None,
                focus: Some(self.focus),
            };
            self.handle.update_accesskit(update);
            return true;
        }
        false
    }

    fn size(&mut self, size: Size) {
        self.size = size;
    }

    fn got_focus(&mut self) {
        self.update_focus(true);
    }

    fn lost_focus(&mut self) {
        self.update_focus(false);
    }

    fn request_close(&mut self) {
        self.handle.close();
    }

    fn destroy(&mut self) {
        Application::global().quit()
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

fn main() {
    tracing_subscriber::fmt().init();

    let app = Application::new().unwrap();
    let mut builder = WindowBuilder::new(app.clone());
    builder.set_handler(Box::new(HelloState::default()));
    builder.set_title(WINDOW_TITLE);

    let window = builder.build().unwrap();
    window.show();

    app.run(None);
}
