import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/app_state.dart';
import '../services/transaction_service.dart';
import '../services/qr_service.dart';
import 'send_transaction_screen.dart';
import 'receive_screen.dart';
import 'transaction_history_screen.dart';
import 'contacts_screen.dart';

class WalletScreen extends StatefulWidget {
  const WalletScreen({super.key});

  @override
  State<WalletScreen> createState() => _WalletScreenState();
}

class _WalletScreenState extends State<WalletScreen> {
  final _transactionService = TransactionService.instance;
  final _qrService = QrService.instance;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadData();
  }

  Future<void> _loadData() async {
    setState(() {
      _isLoading = true;
    });

    try {
      final appState = Provider.of<AppState>(context, listen: false);

      // Mock address for development
      const address = '0x1234567890abcdef1234567890abcdef12345678';
      appState.setCurrentAddress(address);

      // Get balance
      final balance = await _transactionService.getBalance(address);
      appState.updateBalance(balance);

      // Get transaction history
      final transactions = await _transactionService.getTransactionHistory(
        address,
      );
      appState.updateTransactions(transactions);
    } catch (e) {
      debugPrint('Error loading wallet data: $e');
    } finally {
      setState(() {
        _isLoading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Wallet'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadData,
            tooltip: 'Refresh',
          ),
        ],
      ),
      body:
          _isLoading
              ? const Center(child: CircularProgressIndicator())
              : _buildWalletContent(),
    );
  }

  Widget _buildWalletContent() {
    return Consumer<AppState>(
      builder: (context, appState, child) {
        return RefreshIndicator(
          onRefresh: _loadData,
          child: SingleChildScrollView(
            physics: const AlwaysScrollableScrollPhysics(),
            padding: const EdgeInsets.all(16.0),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Balance card
                _buildBalanceCard(appState),

                const SizedBox(height: 24),

                // Action buttons
                _buildActionButtons(),

                const SizedBox(height: 24),

                // Recent transactions
                _buildRecentTransactions(appState),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildBalanceCard(AppState appState) {
    return Card(
      elevation: 4,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      child: Padding(
        padding: const EdgeInsets.all(20.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Current Balance',
              style: TextStyle(fontSize: 16, fontWeight: FontWeight.w500),
            ),
            const SizedBox(height: 8),
            Row(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                Text(
                  '${appState.balance.toStringAsFixed(4)}',
                  style: const TextStyle(
                    fontSize: 32,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(width: 4),
                const Text(
                  'SEBURE',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.w500),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Text(
              'Address: ${_truncateAddress(appState.currentAddress)}',
              style: const TextStyle(fontSize: 14, color: Colors.grey),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildActionButtons() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: [
        _buildActionButton(
          icon: Icons.send,
          label: 'Send',
          onTap: () {
            Navigator.push(
              context,
              MaterialPageRoute(
                builder: (context) => const SendTransactionScreen(),
              ),
            );
          },
        ),
        _buildActionButton(
          icon: Icons.qr_code,
          label: 'Receive',
          onTap: () {
            Navigator.push(
              context,
              MaterialPageRoute(builder: (context) => const ReceiveScreen()),
            );
          },
        ),
        _buildActionButton(
          icon: Icons.history,
          label: 'History',
          onTap: () {
            Navigator.push(
              context,
              MaterialPageRoute(
                builder: (context) => const TransactionHistoryScreen(),
              ),
            );
          },
        ),
        _buildActionButton(
          icon: Icons.contacts,
          label: 'Contacts',
          onTap: () {
            Navigator.push(
              context,
              MaterialPageRoute(builder: (context) => const ContactsScreen()),
            );
          },
        ),
      ],
    );
  }

  Widget _buildActionButton({
    required IconData icon,
    required String label,
    required VoidCallback onTap,
  }) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: Container(
        width: 70,
        padding: const EdgeInsets.symmetric(vertical: 8),
        child: Column(
          children: [
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.primaryContainer,
                borderRadius: BorderRadius.circular(12),
              ),
              child: Icon(
                icon,
                color: Theme.of(context).colorScheme.onPrimaryContainer,
              ),
            ),
            const SizedBox(height: 8),
            Text(label, style: const TextStyle(fontSize: 12)),
          ],
        ),
      ),
    );
  }

  Widget _buildRecentTransactions(AppState appState) {
    final transactions = appState.transactions;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            const Text(
              'Recent Transactions',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            TextButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(
                    builder: (context) => const TransactionHistoryScreen(),
                  ),
                );
              },
              child: const Text('View All'),
            ),
          ],
        ),
        const SizedBox(height: 8),
        transactions.isEmpty
            ? const Center(
              child: Padding(
                padding: EdgeInsets.all(16.0),
                child: Text(
                  'No transactions yet',
                  style: TextStyle(color: Colors.grey),
                ),
              ),
            )
            : ListView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              itemCount: transactions.length > 3 ? 3 : transactions.length,
              itemBuilder: (context, index) {
                final transaction = transactions[index];
                return _buildTransactionItem(transaction);
              },
            ),
      ],
    );
  }

  Widget _buildTransactionItem(Transaction transaction) {
    final isReceived = !transaction.isOutgoing;

    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: ListTile(
        leading: Container(
          padding: const EdgeInsets.all(8),
          decoration: BoxDecoration(
            color:
                isReceived
                    ? Colors.green.withOpacity(0.2)
                    : Colors.red.withOpacity(0.2),
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(
            isReceived ? Icons.arrow_downward : Icons.arrow_upward,
            color: isReceived ? Colors.green : Colors.red,
          ),
        ),
        title: Text(
          isReceived ? 'Received' : 'Sent',
          style: const TextStyle(fontWeight: FontWeight.bold),
        ),
        subtitle: Text(
          '${transaction.timestamp.day}/${transaction.timestamp.month}/${transaction.timestamp.year}',
        ),
        trailing: Text(
          '${isReceived ? '+' : '-'}${transaction.amount.toStringAsFixed(4)} SEBURE',
          style: TextStyle(
            fontWeight: FontWeight.bold,
            color: isReceived ? Colors.green : Colors.red,
          ),
        ),
      ),
    );
  }

  String _truncateAddress(String address) {
    if (address.length <= 14) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }
}
