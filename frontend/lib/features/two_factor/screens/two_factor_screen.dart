import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/features/two_factor/providers/two_factor_provider.dart';

class TwoFactorScreen extends ConsumerWidget {
  const TwoFactorScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final statusAsync = ref.watch(twoFactorStatusProvider(null));

    return Scaffold(
      appBar: AppBar(
        title: const Text('二要素認証'),
      ),
      body: statusAsync.when(
        data: (status) {
          final isEnabled = status['enabled'] as bool? ?? false;

          return ListView(
            children: [
              ListTile(
                title: const Text('二要素認証'),
                subtitle: const Text('アカウントのセキュリティを強化します'),
                trailing: Switch(
                  value: isEnabled,
                  onChanged: (bool value) async {
                    if (value) {
                      await _showEnableDialog(context, ref);
                    } else {
                      await _showDisableDialog(context, ref);
                    }
                  },
                ),
              ),
              if (isEnabled)
                const Padding(
                  padding: EdgeInsets.all(16),
                  child: Text(
                    '二要素認証が有効になっています。ログイン時に認証コードを入力する必要があります。',
                    style: TextStyle(color: Colors.green),
                  ),
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

  Future<void> _showEnableDialog(BuildContext context, WidgetRef ref) async {
    final result = await ref.read(twoFactorActionsProvider).registerTwoFactor();
    final qrCodeUrl = result['qrCodeUrl'] as String?;
    final secret = result['secret'] as String?;

    if (context.mounted) {
      await showDialog(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('二要素認証を有効にする'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                if (qrCodeUrl != null)
                  Image.network(qrCodeUrl),
                const SizedBox(height: 16),
                Text('シークレット: $secret'),
                const SizedBox(height: 16),
                const Text('認証アプリでQRコードをスキャンし、表示される認証コードを入力してください。'),
                const SizedBox(height: 16),
                TextField(
                  decoration: const InputDecoration(
                    labelText: '認証コード',
                    hintText: '6桁のコード',
                  ),
                  keyboardType: TextInputType.number,
                  maxLength: 6,
                  onSubmitted: (value) async {
                    await ref.read(twoFactorActionsProvider).enableTwoFactor(value);
                    if (context.mounted) {
                      Navigator.of(context).pop();
                    }
                  },
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('キャンセル'),
            ),
          ],
        ),
      );
    }
  }

  Future<void> _showDisableDialog(BuildContext context, WidgetRef ref) async {
    final passwordController = TextEditingController();

    return showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('二要素認証を無効にする'),
        content: TextField(
          controller: passwordController,
          decoration: const InputDecoration(
            labelText: 'パスワード',
            hintText: 'パスワードを入力',
          ),
          obscureText: true,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('キャンセル'),
          ),
          TextButton(
            onPressed: () async {
              final password = passwordController.text.trim();
              if (password.isEmpty) return;

              await ref.read(twoFactorActionsProvider).disableTwoFactor(password);

              if (context.mounted) {
                Navigator.of(context).pop();
              }
            },
            style: TextButton.styleFrom(foregroundColor: Colors.red),
            child: const Text('無効にする'),
          ),
        ],
      ),
    );
  }
}
