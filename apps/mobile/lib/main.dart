import 'package:flutter/material.dart';
import 'screens/activity_screen.dart';
import 'screens/character_screen.dart';
import 'screens/commitments_screen.dart';
import 'screens/dashboard_screen.dart';
import 'screens/settings_screen.dart';
import 'services/forgex_api_service.dart';
import 'services/forgex_ffi_service.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final apiService = ForgeXApiService();
  await apiService.init();

  final ffiService = ForgeXFfiService();
  ffiService.init();

  runApp(ForgeXApp(apiService: apiService, ffiService: ffiService));
}

class ForgeXApp extends StatelessWidget {
  final ForgeXApiService apiService;
  final ForgeXFfiService ffiService;

  const ForgeXApp({
    super.key,
    required this.apiService,
    required this.ffiService,
  });

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'ForgeX',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        brightness: Brightness.dark,
        scaffoldBackgroundColor: const Color(0xFF121216),
        colorScheme: const ColorScheme.dark(
          primary: Colors.cyan,
          secondary: Colors.cyanAccent,
          surface: Color(0xFF1E1E24),
        ),
        appBarTheme: const AppBarTheme(
          backgroundColor: Color(0xFF16161C),
          elevation: 0,
          centerTitle: false,
        ),
        bottomNavigationBarTheme: const BottomNavigationBarThemeData(
          backgroundColor: Color(0xFF16161C),
          selectedItemColor: Colors.cyan,
          unselectedItemColor: Colors.grey,
          type: BottomNavigationBarType.fixed,
        ),
      ),
      home: MainNavigationShell(apiService: apiService),
    );
  }
}

class MainNavigationShell extends StatefulWidget {
  final ForgeXApiService apiService;

  const MainNavigationShell({super.key, required this.apiService});

  @override
  State<MainNavigationShell> createState() => _MainNavigationShellState();
}

class _MainNavigationShellState extends State<MainNavigationShell> {
  int _currentIndex = 0;

  late final List<Widget> _screens;

  @override
  void initState() {
    super.initState();
    _screens = [
      DashboardScreen(apiService: widget.apiService),
      CommitmentsScreen(apiService: widget.apiService),
      ActivityScreen(apiService: widget.apiService),
      CharacterScreen(apiService: widget.apiService),
      SettingsScreen(apiService: widget.apiService),
    ];
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: IndexedStack(
        index: _currentIndex,
        children: _screens,
      ),
      bottomNavigationBar: BottomNavigationBar(
        currentIndex: _currentIndex,
        onTap: (index) => setState(() => _currentIndex = index),
        items: const [
          BottomNavigationBarItem(
            icon: Icon(Icons.dashboard_outlined),
            activeIcon: Icon(Icons.dashboard),
            label: 'Dashboard',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.check_circle_outline),
            activeIcon: Icon(Icons.check_circle),
            label: 'Promises',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.timer_outlined),
            activeIcon: Icon(Icons.timer),
            label: 'Activity',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.shield_outlined),
            activeIcon: Icon(Icons.shield),
            label: 'Character',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.settings_outlined),
            activeIcon: Icon(Icons.settings),
            label: 'Settings',
          ),
        ],
      ),
    );
  }
}
