import 'package:flutter/foundation.dart';

/// Contact model for the wallet
class Contact {
  /// Unique identifier for the contact
  final String id;

  /// Name of the contact
  final String name;

  /// Blockchain address of the contact
  final String address;

  /// Optional notes about the contact
  final String? notes;

  /// Create a new contact
  Contact({
    required this.id,
    required this.name,
    required this.address,
    this.notes,
  });

  /// Create a contact from JSON
  factory Contact.fromJson(Map<String, dynamic> json) {
    return Contact(
      id: json['id'] as String,
      name: json['name'] as String,
      address: json['address'] as String,
      notes: json['notes'] as String?,
    );
  }

  /// Convert contact to JSON
  Map<String, dynamic> toJson() {
    return {'id': id, 'name': name, 'address': address, 'notes': notes};
  }
}

/// Contact list provider for state management
class ContactListProvider extends ChangeNotifier {
  List<Contact> _contacts = [];

  /// Get all contacts
  List<Contact> get contacts => List.unmodifiable(_contacts);

  /// Add a new contact
  void addContact(Contact contact) {
    _contacts.add(contact);
    notifyListeners();
  }

  /// Update an existing contact
  void updateContact(Contact contact) {
    final index = _contacts.indexWhere((c) => c.id == contact.id);
    if (index != -1) {
      _contacts[index] = contact;
      notifyListeners();
    }
  }

  /// Delete a contact
  void deleteContact(String id) {
    _contacts.removeWhere((contact) => contact.id == id);
    notifyListeners();
  }

  /// Find a contact by address
  Contact? findByAddress(String address) {
    try {
      return _contacts.firstWhere((contact) => contact.address == address);
    } catch (e) {
      return null;
    }
  }

  /// Load contacts from storage
  void loadContacts(List<Contact> contacts) {
    _contacts = contacts;
    notifyListeners();
  }
}
