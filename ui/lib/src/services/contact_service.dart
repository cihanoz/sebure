import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../models/contact.dart';

/// Service for managing contacts
class ContactService {
  // Singleton instance
  static final ContactService _instance = ContactService._internal();
  static ContactService get instance => _instance;

  // Private constructor for singleton
  ContactService._internal();

  // Storage key for contacts
  static const String _storageKey = 'sebure_contacts';

  // Track if the service is initialized
  bool _isInitialized = false;

  // Contact list provider
  final ContactListProvider _contactListProvider = ContactListProvider();
  ContactListProvider get contactListProvider => _contactListProvider;

  /// Initialize the contact service
  Future<bool> initialize() async {
    if (_isInitialized) return true;

    debugPrint('Initializing contact service...');
    try {
      // Load contacts from storage
      final contacts = await _loadContactsFromStorage();
      _contactListProvider.loadContacts(contacts);

      _isInitialized = true;
      debugPrint('Contact service initialized successfully');
      return true;
    } catch (e) {
      debugPrint('Error initializing contact service: $e');
      return false;
    }
  }

  /// Add a new contact
  Future<bool> addContact(Contact contact) async {
    if (!_isInitialized) {
      await initialize();
    }

    try {
      _contactListProvider.addContact(contact);
      await _saveContactsToStorage();
      return true;
    } catch (e) {
      debugPrint('Error adding contact: $e');
      return false;
    }
  }

  /// Update an existing contact
  Future<bool> updateContact(Contact contact) async {
    if (!_isInitialized) {
      await initialize();
    }

    try {
      _contactListProvider.updateContact(contact);
      await _saveContactsToStorage();
      return true;
    } catch (e) {
      debugPrint('Error updating contact: $e');
      return false;
    }
  }

  /// Delete a contact
  Future<bool> deleteContact(String id) async {
    if (!_isInitialized) {
      await initialize();
    }

    try {
      _contactListProvider.deleteContact(id);
      await _saveContactsToStorage();
      return true;
    } catch (e) {
      debugPrint('Error deleting contact: $e');
      return false;
    }
  }

  /// Load contacts from storage
  Future<List<Contact>> _loadContactsFromStorage() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final contactsJson = prefs.getString(_storageKey);

      if (contactsJson == null) {
        return [];
      }

      final List<dynamic> contactsList = jsonDecode(contactsJson);
      return contactsList
          .map((json) => Contact.fromJson(json as Map<String, dynamic>))
          .toList();
    } catch (e) {
      debugPrint('Error loading contacts from storage: $e');
      return [];
    }
  }

  /// Save contacts to storage
  Future<bool> _saveContactsToStorage() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      final contactsJson = jsonEncode(
        _contactListProvider.contacts.map((c) => c.toJson()).toList(),
      );
      return await prefs.setString(_storageKey, contactsJson);
    } catch (e) {
      debugPrint('Error saving contacts to storage: $e');
      return false;
    }
  }

  /// Generate a unique ID for a new contact
  String generateContactId() {
    return DateTime.now().millisecondsSinceEpoch.toString();
  }
}
