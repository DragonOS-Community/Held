use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// 编辑命令
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct EditCommand {
    // 删除动作，如果有的话
    action_delete: Option<Action>,

    // 插入操作，如果有的话
    action_insert: Option<Action>,

    // 对于第几行进行的操作（目前就只实现针对单独一个文件的）
    line_number: u16,
}

impl Default for EditCommand {
    fn default() -> Self {
        Self {
            action_delete: None,
            action_insert: None,
            line_number: 0,
        }
    }
}

impl EditCommand {
    pub fn new(action_delete: Action, action_insert: Action, line_number: u16) -> Self {
        Self {
            action_delete: Some(action_delete),
            action_insert: Some(action_insert),
            line_number,
        }
    }

    /// 处理命令，返回对应的反向命令
    pub fn process(&self) -> Result<EditCommand, ()> {
        let mut content_to_insert = String::new();
        let mut content_to_delete = String::new();
        let action_insert;
        let action_delete;
        if let Action::Delete(s) = self.action_delete.as_ref().unwrap() {
            content_to_insert = s.clone();
        }
        if let Action::Insert(s) = self.action_insert.as_ref().unwrap() {
            content_to_delete = s.clone();
        }
        action_delete = Action::Delete(content_to_delete);
        action_insert = Action::Insert(content_to_insert);

        let res = EditCommand::new(action_delete, action_insert, self.line_number);

        Ok(res)
    }

    /// 拿到这命令里的删除内容用于删除
    pub fn get_delete_content(&self) -> Option<String> {
        if let Some(Action::Delete(s)) = self.action_delete.as_ref() {
            return Some(s.clone());
        }
        None
    }

    /// 拿到这命令里的插入内容用于重新插入
    pub fn get_insert_content(&self) -> Option<String> {
        if let Some(Action::Insert(s)) = self.action_insert.as_ref() {
            return Some(s.clone());
        }
        None
    }
}

/// 编辑动作
#[allow(unused)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Action {
    // 插入内容
    Insert(String),

    // 删除内容
    Delete(String),

    // 移动(未实现)
    Move,

    // 其他动作
    None,
}

#[derive(Debug)]
pub struct UndoTreeNode {
    // 父节点
    parent: Weak<RefCell<UndoTreeNode>>,
    // 子节点
    children: Vec<Rc<RefCell<UndoTreeNode>>>,
    // 命令
    command: EditCommand,
    // 指向自身的弱引用
    self_pointer: Weak<RefCell<UndoTreeNode>>,
    // 指向UndoTree的弱引用
    tree_pointer: Weak<RefCell<UndoTree>>,
    // 是不是移动操作（未实现）
    // is_move: bool,
    // 指向上一次操作的指针，用于另一种撤销方式
    last_operation_pointer: Option<Node>,
    // 指向下一次操作的指针，用于另一种撤销方式
    next_operation_pointer: Option<Node>,
}

// impl Clone for UndoTreeNode {
//     fn clone(&self) -> Self {
//         Self {
//             parent: self.parent.clone(),
//             children: self.children.clone(),
//             command: self.command.clone(),
//             self_pointer: self.self_pointer.clone(),
//             tree_pointer: self.tree_pointer.clone(),
//         }
//     }
// }

impl UndoTreeNode {
    pub fn new(command: EditCommand) -> Node {
        let result = Rc::new(RefCell::new(UndoTreeNode {
            parent: Weak::new(),
            children: Vec::new(),
            command,
            self_pointer: Weak::new(),
            tree_pointer: Weak::new(),
            last_operation_pointer: None,
            next_operation_pointer: None,
        }));
        result.borrow_mut().self_pointer = Rc::downgrade(&result);
        result.borrow_mut().parent = Rc::downgrade(&result);
        result
    }

    pub fn get_command(&self) -> EditCommand {
        self.command.clone()
    }

    pub fn set_parent(&mut self, parent: &Node) {
        self.parent = Rc::downgrade(parent);
    }

    pub fn set_tree_pointer(&mut self, root: &Rc<RefCell<UndoTree>>) {
        self.tree_pointer = Rc::downgrade(root);
    }

    pub fn insert_with_command(&mut self, command: EditCommand) {
        let new_node = UndoTreeNode::new(command);
        self.insert(new_node);
    }

    /// 插入子节点
    pub fn insert(&mut self, node: Node) {
        node.borrow_mut().parent = self.self_pointer.clone();
        node.borrow_mut()
            .set_tree_pointer(&self.tree_pointer.upgrade().unwrap());
        let self_pointer = self.self_pointer.upgrade().unwrap().clone();
        node.borrow_mut().last_operation_pointer = Some(self_pointer);
        self.next_operation_pointer = Some(node.clone());
        self.children.push(node);
    }

    /// 删除自身以及子节点
    pub fn delete(&mut self) {
        for child in &self.children {
            child.borrow_mut().delete();
        }
        if let Some(parent) = self.parent.upgrade() {
            match parent.borrow().next_operation_pointer {
                Some(ref next_node)
                    if Rc::ptr_eq(next_node, &self.self_pointer.upgrade().unwrap()) =>
                {
                    parent.borrow_mut().next_operation_pointer = None;
                }
                _ => {}
            }
            parent
                .borrow_mut()
                .children
                .retain(|x| !Rc::ptr_eq(x, &self.self_pointer.upgrade().unwrap()));
        }
    }

    pub fn get_parent(&self) -> Option<Node> {
        self.parent.upgrade()
    }

    pub fn get_children(&self) -> Vec<Node> {
        self.children.clone()
    }
}

#[allow(dead_code)]
#[derive(Debug)]
/// 撤销树
/// 用于记录编辑历史，实现撤销和恢复功能
/// 支持两种撤销方式，可以根据用户的需要来选择哪种撤销方式
/// 此撤销树仅用于记录和返回相关操作，相对独立于其他部分
pub struct UndoTree {
    /// 根节点
    root: Node,
    /// 当前节点
    current_node: Node,
    /// 待撤销的节点，代表操作
    to_undo: Vec<Node>,
    /// 待恢复的节点，代表操作
    to_redo: Vec<Node>,
}

type Node = Rc<RefCell<UndoTreeNode>>;

#[allow(dead_code)]
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

    pub fn get_root(&self) -> Node {
        self.root.clone()
    }

    pub fn get_current_node(&self) -> Node {
        self.current_node.clone()
    }

    pub fn set_current_node(&mut self, node: Node) {
        self.current_node = node;
    }

    fn get_to_redo(&mut self) -> Result<EditCommand, ()> {
        match self.to_redo.pop() {
            Some(node) => Ok(node.borrow().get_command()),
            None => Err(()),
        }
    }

    fn get_to_undo(&mut self) -> Result<EditCommand, ()> {
        match self.to_undo.pop() {
            Some(node) => Ok(node.borrow().get_command().process()?),
            None => Err(()),
        }
    }

    /// 插入新操作，生成新节点
    pub fn push(&mut self, new_node: Node) {
        self.current_node.borrow_mut().insert(new_node.clone());
        self.current_node = new_node;
        self.to_undo.push(self.current_node.clone());
    }

    pub fn push_with_command(&mut self, command: EditCommand) {
        let new_node = UndoTreeNode::new(command);
        self.push(new_node);
    }

    /// 撤销操作，返回对应的反向操作
    pub fn undo(&mut self) -> Result<EditCommand, ()> {
        match self.get_to_undo() {
            Ok(command) => {
                let current_node = self.current_node.clone();
                if let Some(parent) = current_node.borrow().get_parent() {
                    self.to_redo.push(self.current_node.clone());
                    self.current_node = parent;
                }
                Ok(command)
            }
            Err(e) => Err(e),
        }
    }

    /// 恢复操作，返回对应的正向操作
    pub fn redo(&mut self) -> Result<EditCommand, ()> {
        match self.get_to_redo() {
            Ok(command) => {
                let current_node = self.current_node.borrow();
                if let Some(child) = current_node.get_children().first().cloned() {
                    drop(current_node);
                    self.current_node = child;
                    self.to_undo.push(self.current_node.clone());
                }
                Ok(command)
            }
            Err(e) => Err(e),
        }
    }

    /// 另一种撤销方式，会跟踪上一次操作的指针，返回对应的反向操作
    pub fn undo_last_operation(&mut self) -> Result<EditCommand, ()> {
        let last_node = self.current_node.borrow().last_operation_pointer.clone();
        if let Some(node) = last_node {
            let command = self.current_node.borrow().get_command();
            node.borrow_mut().next_operation_pointer = Some(self.current_node.clone());
            self.current_node.borrow_mut().last_operation_pointer = Some(node.clone());
            self.current_node = node;
            Ok(command.process()?)
        } else {
            Err(())
        }
    }

    /// 另一种撤销方式，会跟踪下一次操作的指针，返回对应的正向操作
    pub fn redo_next_operation(&mut self) -> Result<EditCommand, ()> {
        let next_node = self.current_node.borrow().next_operation_pointer.clone();
        if let Some(node) = next_node {
            let command = self.current_node.borrow().get_command();
            self.current_node = node;
            Ok(command)
        } else {
            Err(())
        }
    }
}
