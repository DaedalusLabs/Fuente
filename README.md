# Fuente

Cargo workspace for development of Fuente apps. 
Apps are built using the [Yew](https://yew.rs/) framework, which compiles to WASM and runs on browsers.
Manifest files allow for PWA functionality on supported devices, and a Service Worker handles 
local caching to save bandwith on the heavy WASM binaries.
User Interface is built using [TailwindCSS](https://tailwindcss.com/).
Shared models, UI features, contexts,and library bindings are available under the `fuente` crate.

## Data Models 

All data used and shared across the apps is stored and trasmitted as Nostr `SignedNotes`. This allows for 
safe sharing of data using encryption standards like [NIP-04](https://nostr-nips.com/nip-04), as well as client-side validation. 

## Apps 

Apps are built as separate crates. Each app is a standalone Yew app, which compiles to a set of static WASM, JS, and CSS files
which can then be served to users. Apps  rely on a series of relays to transmit data, which is managed through a `RelayPool`.

## Services 

Services are built as separate crates. Each service is a standalone Rust app, which can be run as a standalone binary.
These services share a Nostr keypair, which can be considered the Fuente identity, and allows them to interact with the Nostr network. 
Services can be run on the same machine, or on different machines, and can be scaled horizontally to handle more traffic.

### Invoicer

Service for coordinating payments between Fuente users and businesses. 
The service litenes to the Nost relays for order requests from the user application.
Once it receives an request, it will request a corresponding invoice from the businesses LN URL, and create a HODL invoice 
to show to the user. When the HODL invoice is paid, the service will notify the business the order has been requested succesfuly.
if the business accepts the order, the invoicer will pay the business invoice, 
settle the HODL invoice, and notify the user the order has been accepted.

### DeliveryBot

Posts delivery states to coordinate businesses, drivers, and users, to finalize the delivery of purchased orders.
Business can notify when an order is ready for pickup. The driver application will recieve these notifications,
and will be able to accept the order. Once the driver accepts the order, the user will be notified the order is on its way.
Once a driver has delivered the order, they can notify the service, which will updat the order status to completed on the relays.

### RatingBot

Listens for completed orders (both delivered and cancelled) to provide ratings and metrics on both businesses and drivers.
