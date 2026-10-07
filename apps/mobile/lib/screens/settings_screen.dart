import 'package:flutter/material.dart';
import '../services/forgex_api_service.dart';

class SettingsScreen extends StatefulWidget {
  final ForgeXApiService apiService;

  const SettingsScreen({super.key, required this.apiService});

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  late final TextEditingController _urlCtrl;
  late final TextEditingController _tokenCtrl;
  bool? _isConnected;
  bool _isTesting = false;

  @override
  void initState() {
    super.initState();
    _urlCtrl = TextEditingController(text: widget.apiService.serverUrl);
    _tokenCtrl = TextEditingController(text: widget.apiService.pairingToken);
    _testConnection();
  }

  Future<void> _testConnection() async {
    setState(() => _isTesting = true);
    final ok = await widget.apiService.testConnection();
    if (mounted) {
      setState(() {
        _isConnected = ok;
        _isTesting = false;
      });
    }
  }

  Future<void> _save() async {
    await widget.apiService.saveSettings(_urlCtrl.text, _tokenCtrl.text);
    _testConnection();
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Settings saved')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Pairing & Settings')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            color: const Color(0xFF1E1E24),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Row(
                children: [
                  Icon(
                    _isConnected == true
                        ? Icons.cloud_done
                        : Icons.cloud_off,
                    color: _isConnected == true ? Colors.greenAccent : Colors.redAccent,
                    size: 32,
                  ),
                  const SizedBox(width: 16),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        _isConnected == true ? 'Connected to ForgeX' : 'Disconnected / Standalone',
                        style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
                      ),
                      Text(
                        _isConnected == true
                            ? 'Syncing behavioral policies in real time'
                            : 'Run "forgex serve" on your laptop to pair',
                        style: const TextStyle(fontSize: 12, color: Colors.grey),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 24),
          const Text('Cross-Device Pair Settings', style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16, color: Colors.cyan)),
          const SizedBox(height: 8),
          const Text(
            'Enter the IP address of your laptop where "forgex serve" is currently running.',
            style: TextStyle(fontSize: 12, color: Colors.grey),
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _urlCtrl,
            decoration: const InputDecoration(
              labelText: 'ForgeX Daemon URL',
              hintText: 'e.g. http://192.168.1.50:8080 or http://10.0.2.2:8080',
              filled: true,
              fillColor: Color(0xFF1E1E24),
            ),
          ),
          const SizedBox(height: 16),
          TextField(
            controller: _tokenCtrl,
            decoration: const InputDecoration(
              labelText: 'Pairing Token',
              hintText: 'e.g. fx_a1b2c3d4',
              filled: true,
              fillColor: Color(0xFF1E1E24),
            ),
          ),
          const SizedBox(height: 24),
          Row(
            children: [
              Expanded(
                child: OutlinedButton(
                  onPressed: _isTesting ? null : _testConnection,
                  child: _isTesting
                      ? const SizedBox(height: 16, width: 16, child: CircularProgressIndicator(strokeWidth: 2))
                      : const Text('Test Connection'),
                ),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: ElevatedButton(
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.cyan,
                    foregroundColor: Colors.black,
                  ),
                  onPressed: _save,
                  child: const Text('Save & Connect', style: TextStyle(fontWeight: FontWeight.bold)),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
