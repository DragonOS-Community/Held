// use std::{any::Any, borrow::BorrowMut, cell::RefCell, rc::Weak, sync::{Arc, Mutex, RwLock}};

// use serde::de;


// #[allow(unused)]

// /// 编辑命令
// #[derive(Debug)]
// struct EditCommand {
//     action: Action,
//     params: Vec<String>,
// }

// impl Default for EditCommand {
//     fn default() -> Self {
//         Self {
//             action: Action::Insert(String::new()),
//             params: Vec::new(),
//         }
//     }
// }

// /// 编辑动作
// #[allow(unused)]
// #[derive(Debug)]
// enum Action {
//     Insert(String),
//     Delete(String ,String),// 删除的起始位置和结束位置
//     Replace(String ,String ,String),// 替换的起始位置、结束位置和替换内容
//     Move(String ,String),// 移动的起始位置和结束位置
// }

// #[allow(unused)]
// #[derive(Debug)]
// struct UndoTreeNode {
//     parent: Weak<LockedUndoTreeNode>,
//     children: Vec<Arc<LockedUndoTreeNode>>,
//     commands: EditCommand,
//     self_pointer: Weak<LockedUndoTreeNode>,
//     tree_pointer: Weak<UndoTree>,
// }

// #[allow(unused)]
// impl UndoTreeNode {
//     fn new(command: EditCommand) -> Self {
//         Self {
//             parent: None,
//             children: RwLock::new(Vec::new()),
//             commands: vec![command],
//             self_pointer: Weak::new(),
//         }
//     }

//     fn add_child(&mut self, child: Arc<UndoTreeNode>) {
//         self.children.write().unwrap().push(child.clone());
//         child
//             .as_ref()
//             .borrow_mut()
//             .parent = Some(Arc::new(self))
//     }
// }

// #[derive(Debug)]
// struct LockedUndoTreeNode(Arc<Mutex<UndoTreeNode>>);

// #[allow(unused)]
// struct UndoTree {
//     root: Arc<LockedUndoTreeNode>,
//     current_node: Arc<LockedUndoTreeNode>,
// }

// impl UndoTree {
//     fn root_node(&self) -> Arc<LockedUndoTreeNode> {
//         self.root.clone()
//     }
//     fn as_any_ref(&self) -> &dyn Any {
//         self
//     }
//     pub fn new() -> Arc<Self> {
//         let root: Arc<LockedUndoTreeNode> = Arc::new(LockedUndoTreeNode(Arc::new(Mutex::new(UndoTreeNode{
//             parent: Weak::default(),
//             children: Vec::new(),
//             commands: EditCommand::default(),
//             self_pointer: Weak::default(),
//             tree_pointer: Weak::default(),
//         }))));

//         let result: Arc<UndoTree> = Arc::new(UndoTree {
//             root: root.clone(),
//             current_node: root,
//         });

//         let mut root_guard = result.root.0.lock().unwrap();
//         root_guard.parent = Arc::downgrade(&result.root);
//     }
// }


use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// 编辑命令
#[allow(unused)]
#[derive(Debug,Clone,PartialEq, PartialOrd)]
pub struct EditCommand {
    // 每个命令所执行的动作
    action: Action,

    // 每个命令所需要的参数, 如插入内容、删除的起始位置和结束位置等
    params: Vec<String>,
}

impl Default for EditCommand {
    fn default() -> Self {
        Self {
            action: Action::None,
            params: Vec::new(),
        }
    }
}

#[allow(unused)]
impl EditCommand {
    pub fn new(action: Action, params: Vec<String>) -> Self {
        Self {
            action,
            params,
        }
    }
    /// 处理命令，返回对应的反向命令
    pub fn process(&self) -> Result<EditCommand, ()> {
        let res = match self.action {
            Action::Insert => {
                EditCommand::new(Action::Delete, self.params.clone())
            }
            Action::Delete => {
                EditCommand::new(Action::Insert, self.params.clone())
            }
            Action::Replace => {
                EditCommand::new(Action::Restore, self.params.clone())
            }
            Action::Restore => {
                EditCommand::new(Action::Replace, self.params.clone())
            }
            Action::Move => {
                EditCommand::new(Action::Move, self.params.clone())
            }
            _ => EditCommand::default()
        };

        Ok(res)
    }
}

/// 编辑动作
#[allow(unused)]
#[derive(Debug,Clone,PartialEq, PartialOrd)]
pub enum Action {
    // 在起始位置的插入内容
    Insert,

    // 删除的起始位置和结束位置
    Delete,

    // 替换的起始位置、结束位置和替换内容
    Replace,

    // 恢复的位置和内容
    Restore,

    // 移动的起始位置和结束位置
    Move,

    // 其他动作
    None,
}


#[allow(unused)]
#[derive(Debug)]
pub struct UndoTreeNode {
    parent: Weak<RefCell<UndoTreeNode>>,
    children: Vec<Rc<RefCell<UndoTreeNode>>>,
    command: EditCommand,
    self_pointer: Weak<RefCell<UndoTreeNode>>,
    tree_pointer: Weak<RefCell<UndoTree>>,
}

#[allow(unused)]
impl UndoTreeNode {
    pub fn new(command: EditCommand) -> Rc<RefCell<UndoTreeNode>> {
        let result = Rc::new(RefCell::new(UndoTreeNode {
            parent: Weak::new(),
            children: Vec::new(),
            command,
            self_pointer: Weak::new(),
            tree_pointer: Weak::new(),
        }));
        result.borrow_mut().self_pointer = Rc::downgrade(&result);
        result
    }

    pub fn get_command(&self) -> EditCommand {
        self.command.clone()
    }

    pub fn set_parent(&mut self, parent: &Rc<RefCell<Self>>){
        self.parent = Rc::downgrade(parent);
    }

    pub fn set_tree_pointer(&mut self, root: &Rc<RefCell<UndoTree>>){
        self.tree_pointer = Rc::downgrade(root);
    }

    pub fn insert_with_command(&mut self, command: EditCommand) {
        let new_node = UndoTreeNode::new(command);
        new_node.borrow_mut().parent = self.self_pointer.clone();
        new_node.borrow_mut().tree_pointer = self.tree_pointer.clone();
        self.children.push(new_node.clone());
    }

    pub fn insert(&mut self, node: Rc<RefCell<UndoTreeNode>>) {
        node.borrow_mut().parent = self.self_pointer.clone();
        node.borrow_mut().tree_pointer = self.tree_pointer.clone();
        self.children.push(node.clone());
    }

    pub fn delete_with_command(&mut self, command: EditCommand) {
        self.children
            .remove(self.children.iter().position(|x| x.borrow().get_command() == command).unwrap());
    }

    pub fn delete(&mut self, node: Rc<RefCell<UndoTreeNode>>) {
        self.children
            .remove(self.children.iter().position(|x| x.borrow().get_command() == node.borrow().get_command()).unwrap());
    }

    pub fn get_parent(&self) -> Option<Rc<RefCell<UndoTreeNode>>> {
        self.parent.upgrade()
    }

}

#[allow(unused)]
#[derive(Debug)]
pub struct UndoTree {
    root: Rc<RefCell<UndoTreeNode>>,
    current_node: Rc<RefCell<UndoTreeNode>>,
    to_undo: Vec<Rc<RefCell<UndoTreeNode>>>,
    to_redo: Vec<Rc<RefCell<UndoTreeNode>>>
}

type Node = Rc<RefCell<UndoTreeNode>>;

#[allow(unused)]
impl UndoTree {
    pub fn new(command: EditCommand) -> UndoTree {
        let root = UndoTreeNode::new(command);
        UndoTree {
            root: root.clone(),
            current_node: root,
            to_undo: vec![],
            to_redo: vec![],
        }
    }

    pub fn get_root(&self) -> Rc<RefCell<UndoTreeNode>> {
        self.root.clone()
    }

    pub fn get_current_node(&self) -> Rc<RefCell<UndoTreeNode>> {
        self.current_node.clone()
    }

    pub fn set_current_node(&mut self, node: Rc<RefCell<UndoTreeNode>>) {
        self.current_node = node;
    }

    pub fn get_to_redo(&mut self) -> Result<Node, ()> {
        Ok(
            self
                .to_redo
                .pop()
                .unwrap()
        )
    }

    pub fn get_to_undo(&mut self) -> Result<EditCommand, ()> {
        match self.to_undo.pop().unwrap().borrow().get_command() {
            EditCommand{action: Action::Insert, params} => {

            }
            EditCommand{action: Action::Delete, params} => {}
            EditCommand{action: Action::Move, params} => {}
            EditCommand{action: Action::Replace, params} => {}
            EditCommand{action: Action::Restore, params} => {}
            _ => {}
        };
        Ok(EditCommand::default())
    }

    pub fn add_child_with_command(&mut self, command: EditCommand) {
        self.current_node.borrow_mut().insert_with_command(command)
    }

    pub fn delete_child_with_command(&mut self, command: EditCommand) {
        self.current_node.borrow_mut().delete_with_command(command)
    }

    pub fn insert(&mut self, node: Rc<RefCell<UndoTreeNode>>) {
        self.current_node.borrow_mut().insert(node)
    }

    pub fn delete(&mut self, node: Rc<RefCell<UndoTreeNode>>) {
        self.current_node.borrow_mut().delete(node)
    }

    pub fn push_with_command(&mut self, command: EditCommand) {
        let new_node = UndoTreeNode::new(command);
        self.current_node.borrow_mut().insert(new_node.clone());
        self.current_node = new_node;
        self.to_undo.push(self.current_node.clone());
    }

    pub fn undo(&mut self) -> Result<EditCommand, ()>{
        let current_node = self.current_node.borrow();
        if let Some(parent) = current_node.get_parent() {
            drop(current_node);
            let undo_node = self.current_node.clone();
            self.to_redo.push(self.current_node.clone());
            self.current_node = parent.clone();
            let command = match undo_node.borrow().get_command() {
                EditCommand{action: Action::Insert, params} => {
                    EditCommand{action: Action::Delete, params }
                }
                EditCommand{action: Action::Delete, params} => {
                    EditCommand{action: Action::Insert,params}
                }
                EditCommand{action: Action::Move, params} => {
                    EditCommand{action: Action::Move, params}
                }
                EditCommand{action: Action::Replace, params} => {
                    EditCommand{action: Action::Replace, params}
                }
                EditCommand{action: Action::Restore, params} => {
                    EditCommand{action: Action::Replace, params}
                }
                _ => {EditCommand::default()}
            };

            return Ok(command);
        }

        Ok(EditCommand::default())
    }

    pub fn redo(&mut self) -> Result<EditCommand, ()>{
        let redo_node = self.get_to_redo().unwrap();
        self.current_node = redo_node.clone();
        self.to_undo.push(redo_node.clone());
        let command = redo_node.borrow().get_command();
        Ok(command)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test(){
        let node = UndoTree::new(EditCommand::default()).get_root();
        let command = node.borrow().get_command();
        assert_eq!(command.action, Action::None);
    }
}




