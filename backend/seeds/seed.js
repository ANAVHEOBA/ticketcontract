const dbName = 'ticketing_backend';
const database = db.getSiblingDB(dbName);

const now = new Date();

database.organizers.updateOne(
  { organizer_id: 'org_demo' },
  {
    $setOnInsert: {
      organizer_id: 'org_demo',
      owner_wallet: 'demo_owner_wallet',
      status: 'active',
      updated_at: now,
    },
  },
  { upsert: true }
);

database.events.updateOne(
  { event_id: 'evt_demo' },
  {
    $setOnInsert: {
      event_id: 'evt_demo',
      organizer_id: 'org_demo',
      name: 'Demo Event',
      status: 'active',
      updated_at: now,
    },
  },
  { upsert: true }
);

database.classes.updateOne(
  { class_id: 'class_demo' },
  {
    $setOnInsert: {
      class_id: 'class_demo',
      event_id: 'evt_demo',
      organizer_id: 'org_demo',
      name: 'General',
      status: 'active',
      supply_total: NumberLong(1000),
      supply_reserved: NumberLong(100),
      supply_sold: NumberLong(250),
      updated_at: now,
    },
  },
  { upsert: true }
);

print('mongo seed completed');
