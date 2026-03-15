import 'package:flutter/material.dart';

import '../../shared/workspace_page.dart';

class SettingsPage extends StatelessWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return const WorkspacePage(
      title: '设置',
      subtitle: '管理行为配置、存储策略和后续集成能力。',
      primaryCardTitle: '应用偏好',
      primaryCardBody:
          '下一步将加入默认输出目录、剪贴板自动清空时间、界面偏好和高级功能开关，并保持敏感存储显式可控。',
      sideCardTitle: '平台集成',
      sideCardBody:
          '后续阶段可以接入系统钥匙串、桌面通知、打包配置以及签名更新通道。',
    );
  }
}
