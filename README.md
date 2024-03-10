# Held

DragonOS/Linux Termial text editor

面向DragonOS和Linux系统的终端文本编辑器。


作者：Heyicong  heyicong@dragonos.org

---

## 安装

默认安装路径为``./install/bin/held``

- Linux:  ``make install-linux``
- DragonOS: ``make install-dragonos``

## 使用

**Held**的设计思路是能够使用简单命令实现高效编辑，高效跳转。

Held提供了三个模式：``Command``，``LastLine``和``Insert``

- **Command**

  - ``:``  进入底线（LastLine）模式
  - ``i``  进入插入模式
  - ``f``  标记当前行
  - ``l``  锁定当前行（该行不能被改动/删除）
  - ``q``  跳转到前一个标记行
  - ``w``  跳转到后一个标记行
  - ``a``  跳转到上一个锁定行
  - ``s``  跳转到下一个锁定行
- **LastLine**

  - ``:q``  不保存退出
  - ``:q!``  强制不保存退出
  - ``:wq``  保存退出
  - ``:goto | :gt``  跳转到行或行列
  - ``:flag | :lock`` 批量标记或锁定行
  - ``:unflag | :unlock``  批量取消标记或锁
  - ``:delete | :dl``  批量删除行（锁定行将不被影响）

## 风格

Held支持自定义部分风格，

可以编辑``config.yaml``并且将其放置与Held同目录，Held将会使用你配置的风格。

目前支持的配置：

- ``line``  整体行设置
  - ``number``  行号设置
    - ``enable``  是否启用
    - ``backgroud``  设置行号列背景色
    - ``frontground``  设置行号列前景色
  - ``highlight``  当前行高亮选项
    - ``enable``  是否启用
    - ``color``  高亮颜色
