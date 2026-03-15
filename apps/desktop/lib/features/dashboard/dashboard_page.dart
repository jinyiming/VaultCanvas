import 'package:flutter/material.dart';

import '../../shared/workspace_page.dart';

class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const WorkspacePage(
      title: '安全工作台',
      subtitle: '面向桌面的加解密与隐写工具箱。',
      primaryCardTitle: '产品概览',
      primaryCardBody:
          '可通过左侧导航进入密码生成、文件加密、文件解密、隐写写入和隐写回显等能力。当前桌面端已经具备中文界面、表单交互和核心引擎接入骨架。',
      sideCardTitle: '当前架构',
      sideCardBody:
          'Flutter 负责界面、工作流和任务状态；Rust 负责密码生成、文件加解密、隐写处理和安全文件读写。下一步将接通 Flutter 与 Rust 的实时桥接。',
    );
  }
}
