import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {test} from 'node:test';

import {z} from 'zod';
import {Catalog} from '../dist/catalog/index.js';
import {scrapeSchemaBehavior} from '../dist/resolution/index.js';
import {A2uiMessageSchema} from '../dist/v0_8/index.js';
import {basicCatalog as v09} from '../dist/v0_9/basic_catalog/index.js';
import {basicCatalog as v10} from '../dist/catalogs/basic/v1/index.js';

test('a loaded catalog refuses an oversized array before touching its elements', () => {
  const document = {
    catalogId: 'https://example.com/bounded',
    components: {
      Bounded: {
        type: 'object',
        properties: {
          component: {const: 'Bounded'},
          items: {type: 'array', maxItems: 1, items: {type: 'string'}},
        },
        required: ['component', 'items'],
        additionalProperties: false,
      },
    },
  };
  const schema = Catalog.fromJson(document, '1.0').components.get('Bounded').schema;
  assert.equal(schema.safeParse({items: ['a']}).success, true);

  let reads = 0;
  const items = ['a', 'b'];
  Object.defineProperty(items, 0, {
    get() {
      reads++;
      return 'a';
    },
  });
  const result = schema.safeParse({items});
  assert.equal(result.success, false);
  assert.equal(reads, 0, 'an oversized array must be rejected before elements are read');
  assert.ok(result.error.issues.some(issue => issue.code === 'too_big'));
});

test('native schemas preserve the binder action and strict protocol validation', () => {
  const button = v09.components.get('Button').schema;
  assert.ok(button instanceof z.ZodType);
  assert.deepEqual(scrapeSchemaBehavior(button).shape.action, {type: 'ACTION'});
  assert.equal(z.config().jitless, true);

  assert.equal(A2uiMessageSchema.safeParse({deleteSurface: {surfaceId: 'example'}}).success, true);
  assert.equal(
    A2uiMessageSchema.safeParse({deleteSurface: {surfaceId: 'example', unexpected: true}}).success,
    false,
  );
});

for (const [name, catalog, snapshot] of [
  ['v0.9', v09, 'v0_9/schemas/catalogs/basic/catalog.json'],
  ['v1.0', v10, 'v1_0/schemas/catalogs/basic/catalog.json'],
]) {
  test(`${name} built-in catalog retains the published wire document`, async () => {
    const source = new URL(`../src/${snapshot}`, import.meta.url);
    assert.deepEqual(catalog.toJson(), JSON.parse(await readFile(source, 'utf8')));
  });
}
