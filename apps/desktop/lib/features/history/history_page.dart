import 'package:flutter/material.dart';

import '../../shared/workspace_page.dart';

class HistoryPage extends StatelessWidget {
  const HistoryPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const WorkspacePage(
      title: '历史记录',
      subtitle: '记录操作结果，但不保存敏感信息。',
      primaryCardTitle: '任务日志',
      primaryCardBody:
          '下一步将持久化记录输入输出路径、时间、状态和耗时等元数据，但不会保存密码、密钥或明文内容片段。',
      sideCardTitle: '保留策略',
      sideCardBody:
          '支持手动清理、自动裁剪旧记录，以及导出不含敏感数据的任务摘要，便于排障和审计。',
    );
  }
}
