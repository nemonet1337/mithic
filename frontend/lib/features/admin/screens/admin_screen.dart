import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/admin/providers/admin_provider.dart';

class AdminScreen extends ConsumerWidget {
  const AdminScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final statsAsync = ref.watch(adminStatsProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('管理者'),
      ),
      body: statsAsync.when(
        data: (stats) {
          return ListView(
            children: [
              _buildStatCard(context, 'ユーザー数', stats['usersCount']?.toString() ?? '0', Icons.people),
              _buildStatCard(context, 'ノート数', stats['notesCount']?.toString() ?? '0', Icons.note),
              _buildStatCard(context, 'インスタンス数', stats['instancesCount']?.toString() ?? '0', Icons.public),
              const Divider(),
              _buildSection(
                context,
                'ユーザー管理',
                [
                  _buildTile(
                    context,
                    Icons.search,
                    'ユーザーを検索',
                    () {
                      Navigator.of(context).pushNamed('/admin/users');
                    },
                  ),
                ],
              ),
            ],
          );
        },
        loading: () => const Center(
          child: CircularProgressIndicator(),
        ),
        error: (error, stack) => Center(
          child: Text('エラー: $error'),
        ),
      ),
    );
  }

  Widget _buildStatCard(BuildContext context, String title, String value, IconData icon) {
    return Card(
      margin: const EdgeInsets.all(16),
      child: ListTile(
        leading: Icon(icon),
        title: Text(title),
        trailing: Text(
          value,
          style: const TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
        ),
      ),
    );
  }

  Widget _buildSection(BuildContext context, String title, List<Widget> children) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 24, 16, 8),
          child: Text(
            title,
            style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
          ),
        ),
        ...children,
      ],
    );
  }

  Widget _buildTile(BuildContext context, IconData icon, String title, VoidCallback onTap) {
    return ListTile(
      leading: Icon(icon),
      title: Text(title),
      onTap: onTap,
    );
  }
}
