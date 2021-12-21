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

use std::{any::Any, cell::RefCell, mem::drop, num::NonZeroU64, rc::Rc};

use accesskit::{
    Action, ActionRequest, Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate,
};

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

struct HelloState {
    size: Size,
    focus: NodeId,
    is_window_focused: bool,
}

impl HelloState {
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            size: Default::default(),
            focus: INITIAL_FOCUS,
            is_window_focused: false,
        }))
    }

    fn get_initial_tree(&self) -> TreeUpdate {
        let root = Node {
            children: vec![BUTTON_1_ID, BUTTON_2_ID],
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
            focus: self.is_window_focused.then(|| self.focus),
        }
    }
}

struct HelloHandler {
    state: Rc<RefCell<HelloState>>,
    handle: WindowHandle,
}

impl HelloHandler {
    fn new(state: Rc<RefCell<HelloState>>) -> Self {
        Self {
            state,
            handle: WindowHandle::default(),
        }
    }

    fn update_focus(&self, is_window_focused: bool) {
        let mut state = self.state.borrow_mut();
        state.is_window_focused = is_window_focused;
        self.handle.update_accesskit_if_active(|| TreeUpdate {
            clear: None,
            nodes: vec![],
            tree: None,
            focus: is_window_focused.then(|| state.focus),
        });
    }
}

impl WinHandler for HelloHandler {
    fn connect(&mut self, handle: &WindowHandle) {
        self.handle = handle.clone();
    }

    fn prepare_paint(&mut self) {}

    fn paint(&mut self, piet: &mut piet_common::Piet, _: &Region) {
        let rect = self.state.borrow().size.to_rect();
        piet.fill(rect, &BG_COLOR);
    }

    fn key_down(&mut self, event: KeyEvent) -> bool {
        if event.key == KbKey::Tab {
            let mut state = self.state.borrow_mut();
            state.focus = if state.focus == BUTTON_1_ID {
                BUTTON_2_ID
            } else {
                BUTTON_1_ID
            };
            drop(state);
            self.update_focus(true);
            return true;
        }
        if event.key == KbKey::Enter || event.key == KbKey::Character(" ".into()) {
            // This is a pretty hacky way of updating a node.
            // A real GUI framework would have a consistent way
            // of building a node from underlying data.
            let focus = self.state.borrow().focus;
            let node = if focus == BUTTON_1_ID {
                make_button(BUTTON_1_ID, "You pressed button 1")
            } else {
                make_button(BUTTON_2_ID, "You pressed button 2")
            };
            let update = TreeUpdate {
                clear: None,
                nodes: vec![node],
                tree: None,
                focus: Some(focus),
            };
            self.handle.update_accesskit(update);
            return true;
        }
        false
    }

    fn size(&mut self, size: Size) {
        let mut state = self.state.borrow_mut();
        state.size = size;
    }

    fn got_focus(&mut self) {
        self.update_focus(true);
    }

    fn lost_focus(&mut self) {
        self.update_focus(false);
    }

    fn accesskit_action(&mut self, request: ActionRequest) {
        if let ActionRequest {
            action: Action::Focus,
            target,
            data: None,
        } = request
        {
            if target == BUTTON_1_ID || target == BUTTON_2_ID {
                let mut state = self.state.borrow_mut();
                state.focus = target;
                let is_window_focused = state.is_window_focused;
                drop(state);
                self.update_focus(is_window_focused);
            }
        }
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
    let state = HelloState::new();
    builder.set_handler(Box::new(HelloHandler::new(Rc::clone(&state))));
    builder.set_accesskit_factory(Box::new(move || state.borrow().get_initial_tree()));
    builder.set_title(WINDOW_TITLE);

    let window = builder.build().unwrap();
    window.show();

    app.run(None);
}
