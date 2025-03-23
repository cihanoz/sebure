import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/app_state.dart';
import '../services/transaction_service.dart';
import '../services/contact_service.dart';
import '../services/qr_service.dart';
import '../models/contact.dart';

class SendTransactionScreen extends StatefulWidget {
  final String? recipientAddress;

  const SendTransactionScreen({super.key, this.recipientAddress});

  @override
  State<SendTransactionScreen> createState() => _SendTransactionScreenState();
}

class _SendTransactionScreenState extends State<SendTransactionScreen> {
  final _formKey = GlobalKey<FormState>();
  final _addressController = TextEditingController();
  final _amountController = TextEditingController();
  final _transactionService = TransactionService.instance;
  final _contactService = ContactService.instance;
  final _qrService = QrService.instance;

  bool _isLoading = false;
  double _fee = 0.0001; // Default fee
  String? _selectedContactId;
  List<Contact> _contacts = [];

  @override
  void initState() {
    super.initState();
    _loadContacts();
    if (widget.recipientAddress != null) {
      _addressController.text = widget.recipientAddress!;
    }
  }

  @override
  void dispose() {
    _addressController.dispose();
    _amountController.dispose();
    super.dispose();
  }

  Future<void> _loadContacts() async {
    await _contactService.initialize();
    setState(() {
      _contacts = _contactService.contactListProvider.contacts;
    });
  }

  Future<void> _scanQrCode() async {
    // In a real implementation, this would open a QR scanner
    // For now, we'll just set a mock address
    final address = await _qrService.scanQrFromGallery();
    if (address != null && mounted) {
      setState(() {
        _addressController.text = address;
      });
    }
  }

  Future<void> _estimateFee() async {
    try {
      final fee = await _transactionService.estimateFee(
        transactionType: TransactionType.transfer,
        dataSize: 0, // No additional data for simple transfers
      );
      setState(() {
        _fee = fee;
      });
    } catch (e) {
      debugPrint('Error estimating fee: $e');
    }
  }

  Future<void> _sendTransaction() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() {
      _isLoading = true;
    });

    try {
      final appState = Provider.of<AppState>(context, listen: false);
      final amount = double.parse(_amountController.text);
      final recipientAddress = _addressController.text;

      // In a real implementation, we would get the sender's keys from secure storage
      // For now, we'll use mock values
      const senderPublicKey = '0x1234567890abcdef1234567890abcdef12345678';
      const senderPrivateKey = 'mock_private_key';
      const senderShard = 0;
      const recipientShard = 0;

      final txId = await _transactionService.submitTransaction(
        senderPublicKey: senderPublicKey,
        senderPrivateKey: senderPrivateKey,
        senderShard: senderShard,
        recipientAddress: recipientAddress,
        recipientShard: recipientShard,
        amount: amount,
        fee: _fee,
      );

      if (txId != null) {
        // Create a transaction object for the UI
        final transaction = Transaction(
          id: txId,
          amount: amount,
          timestamp: DateTime.now(),
          isOutgoing: true,
          sender: appState.currentAddress,
          recipient: recipientAddress,
          fee: _fee,
        );

        // Add to transaction history
        appState.addTransaction(transaction);

        // Update balance (subtract amount and fee)
        appState.updateBalance(appState.balance - amount - _fee);

        // Show success message
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('Transaction sent successfully'),
              backgroundColor: Colors.green,
            ),
          );
          Navigator.pop(context);
        }
      } else {
        throw Exception('Failed to send transaction');
      }
    } catch (e) {
      debugPrint('Error sending transaction: $e');
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Error: ${e.toString()}'),
            backgroundColor: Colors.red,
          ),
        );
      }
    } finally {
      if (mounted) {
        setState(() {
          _isLoading = false;
        });
      }
    }
  }

  void _selectContact(String contactId) {
    final contact = _contacts.firstWhere((c) => c.id == contactId);
    setState(() {
      _selectedContactId = contactId;
      _addressController.text = contact.address;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Send Transaction')),
      body:
          _isLoading
              ? const Center(child: CircularProgressIndicator())
              : SingleChildScrollView(
                padding: const EdgeInsets.all(16.0),
                child: Form(
                  key: _formKey,
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      // Contact selection
                      if (_contacts.isNotEmpty) ...[
                        const Text(
                          'Select Contact',
                          style: TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                        const SizedBox(height: 8),
                        SizedBox(
                          height: 80,
                          child: ListView.builder(
                            scrollDirection: Axis.horizontal,
                            itemCount: _contacts.length,
                            itemBuilder: (context, index) {
                              final contact = _contacts[index];
                              final isSelected =
                                  _selectedContactId == contact.id;
                              return Padding(
                                padding: const EdgeInsets.only(right: 12),
                                child: GestureDetector(
                                  onTap: () => _selectContact(contact.id),
                                  child: Column(
                                    children: [
                                      CircleAvatar(
                                        radius: 25,
                                        backgroundColor:
                                            isSelected
                                                ? Theme.of(
                                                  context,
                                                ).colorScheme.primary
                                                : Theme.of(
                                                  context,
                                                ).colorScheme.primaryContainer,
                                        child: Text(
                                          contact.name[0].toUpperCase(),
                                          style: TextStyle(
                                            color:
                                                isSelected
                                                    ? Theme.of(
                                                      context,
                                                    ).colorScheme.onPrimary
                                                    : Theme.of(context)
                                                        .colorScheme
                                                        .onPrimaryContainer,
                                            fontWeight: FontWeight.bold,
                                          ),
                                        ),
                                      ),
                                      const SizedBox(height: 4),
                                      Text(
                                        contact.name,
                                        style: TextStyle(
                                          fontWeight:
                                              isSelected
                                                  ? FontWeight.bold
                                                  : FontWeight.normal,
                                        ),
                                      ),
                                    ],
                                  ),
                                ),
                              );
                            },
                          ),
                        ),
                        const SizedBox(height: 16),
                      ],

                      // Recipient address
                      const Text(
                        'Recipient Address',
                        style: TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 8),
                      TextFormField(
                        controller: _addressController,
                        decoration: InputDecoration(
                          border: const OutlineInputBorder(),
                          hintText: 'Enter recipient address',
                          suffixIcon: IconButton(
                            icon: const Icon(Icons.qr_code_scanner),
                            onPressed: _scanQrCode,
                            tooltip: 'Scan QR Code',
                          ),
                        ),
                        validator: (value) {
                          if (value == null || value.isEmpty) {
                            return 'Please enter a recipient address';
                          }
                          if (!_qrService.isValidAddress(value)) {
                            return 'Invalid address format';
                          }
                          return null;
                        },
                      ),
                      const SizedBox(height: 16),

                      // Amount
                      const Text(
                        'Amount',
                        style: TextStyle(
                          fontSize: 16,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const SizedBox(height: 8),
                      TextFormField(
                        controller: _amountController,
                        keyboardType: const TextInputType.numberWithOptions(
                          decimal: true,
                        ),
                        decoration: const InputDecoration(
                          border: OutlineInputBorder(),
                          hintText: 'Enter amount',
                          suffixText: 'SEBURE',
                        ),
                        validator: (value) {
                          if (value == null || value.isEmpty) {
                            return 'Please enter an amount';
                          }
                          try {
                            final amount = double.parse(value);
                            if (amount <= 0) {
                              return 'Amount must be greater than 0';
                            }
                            final appState = Provider.of<AppState>(
                              context,
                              listen: false,
                            );
                            if (amount + _fee > appState.balance) {
                              return 'Insufficient balance';
                            }
                          } catch (e) {
                            return 'Invalid amount format';
                          }
                          return null;
                        },
                        onChanged: (_) => _estimateFee(),
                      ),
                      const SizedBox(height: 16),

                      // Fee
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          const Text(
                            'Transaction Fee',
                            style: TextStyle(
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          Text(
                            '${_fee.toStringAsFixed(6)} SEBURE',
                            style: TextStyle(
                              color: Theme.of(context).colorScheme.primary,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 24),

                      // Send button
                      SizedBox(
                        width: double.infinity,
                        child: ElevatedButton(
                          onPressed: _sendTransaction,
                          style: ElevatedButton.styleFrom(
                            padding: const EdgeInsets.symmetric(vertical: 16),
                            backgroundColor:
                                Theme.of(context).colorScheme.primary,
                            foregroundColor:
                                Theme.of(context).colorScheme.onPrimary,
                          ),
                          child: const Text(
                            'Send Transaction',
                            style: TextStyle(
                              fontSize: 16,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
    );
  }
}
