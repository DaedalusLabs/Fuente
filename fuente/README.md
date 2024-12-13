# Data Model Overview

## Profiles Overview
This section outlines several types of profiles used for securely storing and transmitting personal and business information. Each profile includes essential fields, supports signing, and enables encryption for secure sharing.

### `ConsumerProfile`

Represents a consumer's personal profile, including basic information (nickname, phone, email), and facilitates secure sharing and transmission via signing and encryption.

### `ConsumerAddress`

Stores location-based data (address and coordinates), supporting secure sharing, signing, and encryption to verify and protect address information.


### `CommerceProfile`

Represents a business's profile, including details like business name, contact info, geolocation, and Lightning Network address, while ensuring secure storage and sharing.

### `DriverProfile`

Stores basic driver details (nickname, phone number), supporting systems that require driver profiles, such as ride-sharing or delivery services.


### `CoordinateStrings`

Provides a simple and flexible way to work with coordinates (latitude and longitude), particularly in situations where coordinates need to be serialized or deserialized in string format.

### Common Key Functionalities
*All profiles share the following key functionalities:*

**1. Profile Creation:** Each profile is created using a new() method, which initializes the profile with the required fields (e.g., name, contact info, location, etc.).

**2. Serialization/Deserialization:** Profiles can be converted between different formats (e.g., Rust, JSON, JavaScript) using various traits:

- `ToString` for converting to a string (JSON).

- `From<JsValue>` and `Into<JsValue>` for integration with JavaScript-based systems.

- `TryFrom<String>` for deserialization from JSON.

**3. Signing Data:** The `signed_data()` method signs the profile using the user's private key, ensuring its authenticity and protection against tampering.

**4. Giftwrapping:** The `giftwrapped_data()` method encrypts the profile, adding an extra layer of security for secure sharing with a recipient.

**5. Getter Methods:** Each profile provides getter methods (e.g., `nickname()`, `telephone()`, `email()`, `geolocation()`) to access the profile's information in a "read-only" manner.

**6. Defaults:** The `Default()` method provides preset values for profiles when specific data is not provided (e.g., default address or coordinates).

**7. Conversion from Signed Notes:** Some profiles can be converted from a SignedNote using the `TryFrom<SignedNote>` method, enabling secure transmission and processing.

>**General Takeaways**
>- *Security: All profiles support cryptographic features like signing and encryption to ensure authenticity and privacy.*
>- *Flexibility: They support multiple formats (e.g., JSON, JavaScript) for seamless integration across systems.*
>- *Trust: The profiles are designed for environments requiring data verification and secure data exchange, ensuring reliability and privacy.*

## Orders Overview

This section covers various structs used to manage, track, and process orders i. Each struct offers unique features for managing the different stages of the order lifecycle, from request to fulfillment, payment, and invoicing.

### `OrderRequest`

Represents a consumer’s order request, encapsulating details such as the commerce platform, consumer’s profile, shipping address, and products ordered. It enables the creation, signing, and secure transmission of the order data.

### `OrderStatus`

Tracks and manages the various stages of an order’s lifecycle, such as `Pending`, `Preparing`, `Ready for Delivery`, `In Delivery`, `Completed`, and `Canceled`. It ensures consistent communication of the order’s current state.

### `OrderPaymentStatus`

Represents the payment status of an order, tracking stages like `Payment Pending`, `Payment Received`, `Payment Failed`, and `Payment Success`.

### `OrderInvoiceState`

Handles the state of an order’s invoice, including payment, order status, and courier information. It supports updates and secure transmissions throughout the order lifecycle.

### `ProductOrder`

Represents an order containing multiple `ProductItems`, supporting operations like adding/removing items, counting duplicates, and calculating total prices.


### `DriverStateUpdate`

Designed to handle and track updates related to a driver's state, including their profile and real-time geolocation information.
*It includes methods for signing, encrypting, and decrypting updates.*


### Common Key Functionalities
*All order-related structs share these common functionalities:*

**1. Serialization & Deserialization:** 
- `ToString` converts the struct into a JSON string, allowing easy transmission and storage.

- `TryFrom<String>` enables deserialization from JSON strings for easy integration with external systems or databases.

**2. Default Initialization:** Provides default values for the struct’s fields, enabling quick instantiation without requiring all fields to be specified immediately.

**3. Signatures for Authentication & Integrity:** 
- `sign_request()` (or equivalent methods) signs the order to verify its authenticity and protect the integrity of the data.

- `giftwrapped_request()` (or equivalent methods) encrypts the signed data for secure transmission to the recipient.

**4. Updating Statuses:** Methods like `update_status()` or `update_payment_status()` allow for seamless updates to the order’s state, including payment, fulfillment, or courier details.

**5. Human-Readable Display:** `display()` provides a user-friendly string for the current state or status, making it easy to present to end-users in interfaces or reports.

>### General Takeaways
>- *Security: All structs support cryptographic features like signing and encryption to ensure authenticity and protect sensitive data.-*
>- *Flexibility: They are compatible with various systems through serialization and deserialization, making them easy to integrate with web APIs, databases, and external services.*
>- *Tracking: Each struct is designed to track and communicate important information related to orders, including statuses, payments, and product details, in a clear and structured manner.*
>- *Compatibility: The use of JSON for serialization ensures compatibility across platforms, whether for communication or data storage.*

## Products Overview

This section outlines the main structures that represent different components of an order or product catalog in a system. Each structure serves a unique purpose, but they share common functionalities for managing and processing product data efficiently.


### `ProductSide`

Represents a side of a product in an order (e.g., an optional variant or addition like a side of fries with a meal). It includes the product's identity, position in an order, name, and price.

### `ProductItem`

Represents an item in an order or catalog. It includes essential details such as the product's name, price, description, and category. It can also support optional sides (e.g., toppings or variants).

### `ProductCategory` 

Represents a category of products within the system (e.g., a "Beverages" category in an online store). It organizes products into groups and supports adding or removing items.

### `ProductMenu` 

Represents a collection of product categories. It manages the organization of categories and their associated products, helping structure an entire catalog or menu.

### Common Key Functionalities
*These structures share several core functionalities, allowing for seamless handling of product data:*

**1. Serialization and Deserialization:**
- Serialize to String: Converts the structure into a JSON string. This is useful for storage, network transmission, or logging. It makes the product information easy to transfer or save in databases.
- Deserialize from String: Converts a JSON string back into the structure, allowing data to be retrieved and used by the system (e.g., from an API, database, or file).

**2. Creating and Managing Objects:**
- Create New Product/Category/Menu: Initialize new instances of products, categories, or menus. *For example, a `ProductItem` can be created with a name, price, and category, while a `ProductCategory` groups products under a specific label like "Beverages."*
- Add and Update: Add new products or categories, or update existing ones. For example, you can add variants (sides) to a product or update a product's name or price in a category or menu.

**3. Product Information Retrieval:**

- Retrieve and Sort: Retrieve products or categories in a specific order. Products are listed in the correct sequence, and categories are organized based on an order attribute, ensuring consistency in how data is accessed or displayed.
- Get Product Attributes: Access product details like name, price, description, or category, making it easy to interact with and manipulate the product data.

**4. Extendable for Additional Attributes:** These structures can be extended to handle more complex product attributes, such as multiple price tiers, inventory counts, or images. This flexibility ensures they can scale with business needs.

>### Functionality Takeaways
>- *Data Storage and Transmission: The shared serialization and deserialization capabilities allow for easy saving, retrieving, and sharing of product data in systems like databases or APIs.*
>- *Variant and Customization Support: These structures are ideal for products with optional variations or add-ons (e.g., product sizes, toppings), making it easy to manage product customization.*
>- *Efficient Product Management: The ability to organize products into categories or menus makes it simple to update, add, or remove items in bulk, supporting large inventories or dynamic product offerings.*
>- *Flexible Integration: The structures are designed to integrate with external systems and support decentralized systems, allowing for diverse use cases in modern e-commerce environments.*

## Data Bases / IndexedDB (IDB) Structures Overview

This section provides an overview of various structures that interact with `IndexedDB` for securely storing and managing consumer, driver, commerce, and order-related data in the browser. These structures ensure data integrity and security while facilitating smooth interactions between web applications and local storage.


### `ConsumerAddressIdb`

Stores and manages consumer address data in `IndexedDB`, including marking an address as the default.

### `ConsumerProfileIdb`

Represents and securely stores a consumer's profile, including their public key and signed note, in `IndexedDB`.

### `CommerceProfileIdb`

Stores a commercial profile along with its signed note and public key in `IndexedDB`.

### `DriverProfileIdb`

Manages and stores a driver’s profile data (including public key and signed note) in IndexedDB for offline access.

### `ProductMenuIdb`

Stores a product menu along with a signed note and public key, ensuring integrity and secure storage in IndexedDB.

### `OrderStateIdb`

Tracks and stores the state of an order in IndexedDB, including its order ID and associated metadata.

### Common Key Functionalities
*The following functionalities are common across most of the IDB structures, allowing for efficient management of data in the browser’s local storage (`IndexedDB`):*

**1. Serialization & Deserialization:**

- Serialize to JavaScript: The structures can be converted into JavaScript values (`JsValue`) for use in web applications. This makes them compatible with frontend frameworks and easy to store/retrieve data from IndexedDB.
- Deserialize from JavaScript: Converts JavaScript values back into the respective Rust structures, enabling interactions with the browser storage system.
- Serialization/Deserialization of Signed Notes: The structures support serialization of signed notes (which verify the authenticity of the data) to/from their respective formats (e.g., SignedNote).

**2. Creation & Storage:**

- Create New Instances: The `new()` method is used to create new instances of each structure (e.g., `ConsumerAddressIdb`, `CommerceProfileIdb`, etc.), initializing them with the necessary data, such as profiles, addresses, or orders.
- Save to `IndexedDB`: Each structure provides methods for saving data to the IndexedDB, ensuring data persists across sessions.
- Retrieve from `IndexedDB`: The structures allow for retrieving data stored in the `IndexedDB`, either through direct queries or by accessing unique identifiers.

**3. `IndexedDB` Integration:**

- Configuration Management: Methods like `config()` set the configuration for storing objects in `IndexedDB`, including defining the store and database names.
- Database Upgrades: Structures implement `upgrade_db()` to manage schema upgrades in the `IndexedDB`, ensuring smooth handling of data changes over time.

**4. Key Management:**

- Unique Identifiers: Each structure provides a unique key (`nostr_id or pubkey`) that can be used to access, retrieve, or delete individual records in `IndexedDB`.
Default/Primary Data: Many structures support marking specific records as the "default" (e.g., an address or profile), allowing users to easily manage their preferences.


>### Functionality Takeaways
>- *Persistent Storage: All structures store data securely and persistently in the browser’s IndexedDB, allowing for offline access and ensuring data is retained across sessions.*
>- *Secure Data Handling: Signed notes and public keys are used to ensure data integrity and secure storage of sensitive information (e.g., profiles, addresses, orders).*
>- *Efficient Data Access: These structures offer easy methods to retrieve, update, and delete data, with unique identifiers ensuring precise access to specific records.*
>- *Seamless Integration with Web Applications: The structures support smooth conversion between Rust and JavaScript, enabling them to work seamlessly with modern web-based applications.*
>- *Simplified User Preferences: Features like default address or profile management make it easy for users to manage their preferences in client-side applications.*

## Admin Overview

This section outlines the core components for managing the administrative configuration of an application by enabling secure and efficient management of access control lists (whitelists, blacklists), user registrations, and exchange rates. They ensure that only authorized entities can interact with specific services and allow administrators to manage, update, and validate critical system configurations with cryptographic security. 

### `AdminConfiguration`

Manages the configuration settings for an admin system within the application.

*Responsible for controlling access through whitelists (admin, commerce, couriers), blacklists (consumer), user registrations, and exchange rates.*

### `AdminConfigurationType`

Categorizes the different configuration types (e.g., admin whitelist, consumer blacklist) an admin can modify. 

### `AdminServerRequest`

Handles configuration-related requests from administrators. It securely manages the configuration data (e.g., whitelist updates, exchange rate changes), ensuring that requests are signed and validated for authenticity before being processed.

### `LastSyncTime`

Represents a timestamp that stores the last synchronization time for a particular operation or event.

### Common Key Functionalities 

**1. Default Configuration:** Default values are set when new instances are created (e.g., empty lists for whitelists and blacklists, exchange rate set to 1.0).

**2. Signing and Validation:** Each configuration (e.g., whitelists, blacklists) is signed using the admin’s private key, ensuring authenticity. The system can validate the integrity of these configurations using encryption and cryptographic tags.

**3. Configuration Management:** Admins can modify lists of allowed entities (whitelists) or blocked entities (blacklists). Configuration types can be changed or updated.

**4.Data Conversion:** Admin configurations and types can be converted between strings, numeric values, or serialized formats for easy communication or storage.

**5. Secure Requests:** Configuration changes are submitted via encrypted, signed requests, ensuring only authorized admins can modify the settings.

**6. Exchange Rate Management:** Admins can set or retrieve exchange rates, which are essential for various application functionalities (e.g., currency conversion).


>### Functionality Takeaways
>- *Centralized Access Control: AdminConfiguration enables the central management of whitelists, blacklists, and other critical configuration data (e.g., exchange rates).*
>- *Secure Configuration Handling: All configurations are signed, validated, and securely managed, ensuring that only trusted entities can access or modify critical data.*
>- *Efficient Data Conversion: Supports seamless conversion of configuration types and request data, improving flexibility and type safety.*
>- *Administrator Control: Admins have the power to manage, modify, and validate critical settings, ensuring the right users and entities can access or interact with specific services.*

