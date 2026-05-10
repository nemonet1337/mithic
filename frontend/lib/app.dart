import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:mithic/l10n/app_localizations.dart';
import 'core/router.dart';
import 'core/theme.dart';
import 'features/auth/providers/auth_provider.dart';

class MithicApp extends ConsumerStatefulWidget {
  const MithicApp({super.key});

  @override
  ConsumerState<MithicApp> createState() => _MithicAppState();
}

class _MithicAppState extends ConsumerState<MithicApp> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(authProvider.notifier).initialize();
    });
  }

  @override
  Widget build(BuildContext context) {
    final router = ref.watch(routerProvider);
    final themeMode = ref.watch(themeModeProvider);
    final lightTheme = ref.watch(lightThemeProvider);
    final darkTheme = ref.watch(darkThemeProvider);
    final locale = ref.watch(localeProvider);

    return MaterialApp.router(
      title: 'Mithic',
      debugShowCheckedModeBanner: false,
      theme: lightTheme,
      darkTheme: darkTheme,
      themeMode: themeMode,
      routerConfig: router,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      locale: locale,
    );
  }
}
