use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// 编辑命令
#[allow(dead_code)]
#[derive(Debug,Clone,PartialEq, PartialOrd)]
pub struct EditCommand {
    // 每个命令所执行的动作
    action: Action,

    // 每个命令所需要的参数, 如操作的起始位置[0],结束位置[1],插入内容[2]等
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

#[allow(dead_code)]
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
                EditCommand::new(Action::Move, vec![self.params[1].clone(), self.params[0].clone()])
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


#[allow(dead_code)]
#[derive(Debug)]
pub struct UndoTreeNode {
    // 父节点
    parent: Weak<RefCell<UndoTreeNode>>,
    // 子节点
    children: Weak<RefCell<UndoTreeNode>>,
    // 命令
    command: EditCommand,
    // 指向自身的弱引用
    self_pointer: Weak<RefCell<UndoTreeNode>>,
    // 指向UndoTree的弱引用
    tree_pointer: Weak<RefCell<UndoTree>>,
}

#[allow(dead_code)]
impl UndoTreeNode {
    pub fn new(command: EditCommand) -> Node {
        let result = Rc::new(RefCell::new(UndoTreeNode {
            parent: Weak::new(),
            children: Weak::new(),
            command,
            self_pointer: Weak::new(),
            tree_pointer: Weak::new(),
        }));
        result.borrow_mut().self_pointer = Rc::downgrade(&result);
        result.borrow_mut().parent = Rc::downgrade(&result);
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
        self.children=Rc::downgrade(&new_node);
    }

    /// 插入子节点
    pub fn insert(&mut self, node: Node) {
        node.borrow_mut().parent = self.self_pointer.clone();
        node.borrow_mut().tree_pointer = self.tree_pointer.clone();
        self.children=Rc::downgrade(&node);
    }

    /// 删除自身以及子节点
    pub fn delete(&mut self) {
        match self.children.upgrade() {
            Some(children) => {children.borrow_mut().delete();}
            None => {
                let parent = self.get_parent().unwrap();
                parent.borrow_mut().children = Weak::new();
            },
        }
    }

    pub fn get_parent(&self) -> Option<Node> {
        self.parent.upgrade()
    }

    pub fn get_children(&self) -> Option<Node> {
        self.children.upgrade()
    }

}

#[allow(dead_code)]
#[derive(Debug)]
/// 编辑树
pub struct UndoTree {
    /// 根节点
    root: Node,
    /// 当前节点
    current_node: Node,
    /// 待撤销的节点代表操作
    to_undo: Vec<Node>,
    /// 待恢复的节点代表操作
    to_redo: Vec<Node>
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

    pub fn get_to_redo(&mut self) -> Result<EditCommand, ()> {
        match self.to_redo.pop() {
            Some(node) => Ok(node.borrow().get_command()),
            None => Err(())
        }
    }

    pub fn get_to_undo(&mut self) -> Result<EditCommand, ()> {
        match self.to_undo.pop() {
            Some(node) => Ok(node.borrow().get_command().process()?) ,
            None => Err(())
        }
    }

    /// 插入新操作，生成新节点
    pub fn push(&mut self, new_node: Node) {
        match self.current_node.borrow().children.upgrade() {
            Some(child) => child.borrow_mut().delete(),
            None => {}
        };
        
        self.current_node.borrow_mut().insert(new_node.clone());
        self.current_node = new_node;
        self.to_undo.push(self.current_node.clone());
    }

    pub fn push_with_command(&mut self, command: EditCommand) {
        let new_node = UndoTreeNode::new(command);
        self.push(new_node);
    }

    /// 撤销操作，返回对应的反向操作
    pub fn undo(&mut self) -> Result<EditCommand, ()>{
        let command = self.get_to_undo().unwrap();
        let current_node = self.current_node.borrow();
        let parent = match current_node.get_parent(){
            Some(parent) => parent,
            None => {
                return Ok(EditCommand::default());
            }
        };
        drop(current_node);

        self.to_redo.push(self.current_node.clone());

        self.current_node = parent;

        Ok(command)
    }

    /// 恢复操作，返回对应的正向操作
    pub fn redo(&mut self) -> Result<EditCommand, ()>{
        let command = self.get_to_redo().unwrap();
        let current_node = self.current_node.borrow();
        let child = match current_node.get_children(){
            Some(child) => child,
            None => {
                return Ok(EditCommand::default());
            }
        };
        drop(current_node);

        self.current_node = child;

        self.to_undo.push(self.current_node.clone());
        
        Ok(command)
    }
}





