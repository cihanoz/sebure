import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/app_state.dart';
import '../services/transaction_service.dart';

class TransactionHistoryScreen extends StatefulWidget {
  const TransactionHistoryScreen({super.key});

  @override
  State<TransactionHistoryScreen> createState() =>
      _TransactionHistoryScreenState();
}

class _TransactionHistoryScreenState extends State<TransactionHistoryScreen> {
  final _transactionService = TransactionService.instance;
  bool _isLoading = false;
  String _filterType = 'All'; // 'All', 'Sent', 'Received'

  @override
  void initState() {
    super.initState();
    _refreshTransactions();
  }

  Future<void> _refreshTransactions() async {
    setState(() {
      _isLoading = true;
    });

    try {
      final appState = Provider.of<AppState>(context, listen: false);
      final transactions = await _transactionService.getTransactionHistory(
        appState.currentAddress,
      );
      appState.updateTransactions(transactions);
    } catch (e) {
      debugPrint('Error refreshing transactions: $e');
    } finally {
      setState(() {
        _isLoading = false;
      });
    }
  }

  List<Transaction> _getFilteredTransactions(List<Transaction> transactions) {
    switch (_filterType) {
      case 'Sent':
        return transactions.where((tx) => tx.isOutgoing).toList();
      case 'Received':
        return transactions.where((tx) => !tx.isOutgoing).toList();
      case 'All':
      default:
        return transactions;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Transaction History'),
        actions: [
          PopupMenuButton<String>(
            icon: const Icon(Icons.filter_list),
            tooltip: 'Filter',
            onSelected: (value) {
              setState(() {
                _filterType = value;
              });
            },
            itemBuilder:
                (context) => [
                  const PopupMenuItem(
                    value: 'All',
                    child: Text('All Transactions'),
                  ),
                  const PopupMenuItem(value: 'Sent', child: Text('Sent Only')),
                  const PopupMenuItem(
                    value: 'Received',
                    child: Text('Received Only'),
                  ),
                ],
          ),
        ],
      ),
      body:
          _isLoading
              ? const Center(child: CircularProgressIndicator())
              : Consumer<AppState>(
                builder: (context, appState, child) {
                  final filteredTransactions = _getFilteredTransactions(
                    appState.transactions,
                  );

                  if (filteredTransactions.isEmpty) {
                    return Center(
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(
                            Icons.history,
                            size: 64,
                            color: Colors.grey.withOpacity(0.5),
                          ),
                          const SizedBox(height: 16),
                          Text(
                            'No ${_filterType.toLowerCase()} transactions yet',
                            style: const TextStyle(
                              fontSize: 16,
                              color: Colors.grey,
                            ),
                          ),
                        ],
                      ),
                    );
                  }

                  return RefreshIndicator(
                    onRefresh: _refreshTransactions,
                    child: ListView.builder(
                      padding: const EdgeInsets.all(8.0),
                      itemCount: filteredTransactions.length,
                      itemBuilder: (context, index) {
                        final transaction = filteredTransactions[index];
                        return _buildTransactionItem(transaction);
                      },
                    ),
                  );
                },
              ),
    );
  }

  Widget _buildTransactionItem(Transaction transaction) {
    final isReceived = !transaction.isOutgoing;
    final formattedDate =
        '${transaction.timestamp.day}/${transaction.timestamp.month}/${transaction.timestamp.year} ${transaction.timestamp.hour}:${transaction.timestamp.minute.toString().padLeft(2, '0')}';

    return Card(
      margin: const EdgeInsets.symmetric(vertical: 4, horizontal: 8),
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Container(
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
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        isReceived ? 'Received' : 'Sent',
                        style: const TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        formattedDate,
                        style: TextStyle(fontSize: 12, color: Colors.grey[600]),
                      ),
                    ],
                  ),
                ),
                Text(
                  '${isReceived ? '+' : '-'}${transaction.amount.toStringAsFixed(4)}',
                  style: TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                    color: isReceived ? Colors.green : Colors.red,
                  ),
                ),
                const SizedBox(width: 4),
                const Text('SEBURE', style: TextStyle(fontSize: 12)),
              ],
            ),
            const Divider(height: 24),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        isReceived ? 'From' : 'To',
                        style: TextStyle(fontSize: 12, color: Colors.grey[600]),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        isReceived
                            ? _truncateAddress(transaction.sender)
                            : _truncateAddress(transaction.recipient),
                        style: const TextStyle(
                          fontSize: 14,
                          fontFamily: 'monospace',
                        ),
                      ),
                    ],
                  ),
                ),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Text(
                      'Fee',
                      style: TextStyle(fontSize: 12, color: Colors.grey[600]),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '${transaction.fee.toStringAsFixed(6)} SEBURE',
                      style: const TextStyle(fontSize: 14),
                    ),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Status',
                  style: TextStyle(fontSize: 12, color: Colors.grey[600]),
                ),
                _buildStatusChip(transaction.status),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              'Transaction ID: ${_truncateAddress(transaction.id)}',
              style: TextStyle(fontSize: 12, color: Colors.grey[600]),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildStatusChip(TransactionStatus status) {
    Color chipColor;
    String statusText;
    IconData statusIcon;

    switch (status) {
      case TransactionStatus.pending:
        chipColor = Colors.orange;
        statusText = 'Pending';
        statusIcon = Icons.pending;
        break;
      case TransactionStatus.confirmed:
        chipColor = Colors.green;
        statusText = 'Confirmed';
        statusIcon = Icons.check_circle;
        break;
      case TransactionStatus.failed:
        chipColor = Colors.red;
        statusText = 'Failed';
        statusIcon = Icons.error;
        break;
    }

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: chipColor.withOpacity(0.2),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(statusIcon, size: 14, color: chipColor),
          const SizedBox(width: 4),
          Text(
            statusText,
            style: TextStyle(
              fontSize: 12,
              fontWeight: FontWeight.bold,
              color: chipColor,
            ),
          ),
        ],
      ),
    );
  }

  String _truncateAddress(String address) {
    if (address.length <= 14) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 6)}';
  }
}
