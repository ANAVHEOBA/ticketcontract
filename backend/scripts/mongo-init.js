const dbName = 'ticketing_backend';
const database = db.getSiblingDB(dbName);

const collections = [
  'organizers',
  'events',
  'classes',
  'tickets',
  'listings',
  'financing',
  'disbursements',
  'settlements',
  'loyalty',
  'trust',
  'chain_events',
  'indexer_cursors',
  'admin_audit_logs'
];

collections.forEach((name) => {
  if (!database.getCollectionNames().includes(name)) {
    database.createCollection(name);
  }
});

database.chain_events.createIndex({ signature: 1 }, { unique: true });
database.chain_events.createIndex({ slot: -1 });
database.chain_events.createIndex({ program_id: 1 });
database.indexer_cursors.createIndex({ cursor_name: 1 }, { unique: true });
database.admin_audit_logs.createIndex({ created_at_epoch: -1 });
database.events.createIndex({ organizer_id: 1 });
database.tickets.createIndex({ event_id: 1 });
database.listings.createIndex({ event_id: 1 });

print('mongo init completed');
