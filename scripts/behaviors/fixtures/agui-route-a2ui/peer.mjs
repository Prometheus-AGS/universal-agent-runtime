import assert from 'node:assert/strict';
import { appendFileSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';

const [port, journal] = process.argv.slice(2);
const model = 'undi95/remm-slerp-l2-13b';
const basic = {
  '1.0': 'https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json',
  'v0.9.1': 'https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json',
};
const record = (row) => appendFileSync(journal, JSON.stringify(row) + '\n');
writeFileSync(journal, '', { flag: 'wx' });

createServer(async (request, response) => {
  try {
    assert.equal(request.method, 'POST');
    assert.ok(['/v1/chat/completions', '/chat/completions'].includes(request.url));
    let bytes = '';
    for await (const chunk of request) {
      bytes += chunk;
      assert.ok(bytes.length <= 2 * 1024 * 1024);
    }
    const body = JSON.parse(bytes);
    assert.equal(body.model, model);
    assert.equal(body.stream, true);
    const latest = body.messages.findLast(message => message.role === 'user');
    const input = typeof latest?.content === 'string' ? latest.content : JSON.stringify(latest?.content ?? '');
    const publication = input.match(/Publish a ((?:legacy )?)ticket surface with surfaceId (ticket-[0-9a-f]+) and heading (?:\\)?"(Ticket [0-9a-f]+)(?:\\)?"/);
    const removal = input.match(/Delete the ticket surface (ticket-[0-9a-f]+)\./);
    const note = publication || removal ? null : input.match(/note-[0-9a-f]+/)?.[0];
    assert.ok(publication || removal || note, 'Expected a controlled publication, removal, or click request');
    const afterTool = body.messages.at(-1)?.role === 'tool';
    const kind = afterTool ? 'after-tool' : publication ? 'publish' : removal ? 'delete' : 'action';
    record({ kind, surface: publication?.[2] ?? removal?.[1], note });
    response.writeHead(200, { 'Content-Type': 'text/event-stream' });
    const emit = (delta, finish_reason = null) => response.write('data: ' + JSON.stringify({
      id: 'uar-rc5-a2ui-fixture', object: 'chat.completion.chunk', model, choices: [{ index: 0, delta, finish_reason }],
    }) + '\n\n');
    if ((publication || removal) && !afterTool) {
      assert.ok(body.tools.some(tool => tool.function?.name === 'a2ui_render'));
      const [, legacy, surfaceId, heading] = publication ?? [null, false, removal[1], null];
      const version = legacy ? 'v0.9.1' : '1.0';
      const bind = path => ({ [legacy ? 'path' : '@path']: path });
      const components = [
        { id: 'root', component: 'Column', children: ['heading', 'note', 'confirm'] },
        { id: 'heading', component: 'Text', text: legacy ? heading : bind('/heading') },
        { id: 'note', component: 'TextField', label: 'Note', value: bind('/form/note') },
        { id: 'confirm', component: 'Button', child: 'label', action: { event: { name: 'confirmTicket', context: { note: bind('/form/note') } } } },
        { id: 'label', component: 'Text', text: 'Confirm' },
      ];
      const createSurface = { surfaceId, catalogId: basic[version] };
      const messages = removal
        ? [{ version, deleteSurface: { surfaceId } }]
        : legacy
          ? [{ version, createSurface }, { version, updateComponents: { surfaceId, components } }]
          : [{ version, createSurface: { ...createSurface, components, dataModel: { heading, phase: 'initial' } } },
            { version, updateDataModel: { surfaceId, path: '/phase', value: 'updated' } }];
      emit({ role: 'assistant', tool_calls: [{
        index: 0, id: `fixture_${surfaceId}`, type: 'function',
        function: { name: 'a2ui_render', arguments: JSON.stringify({ messages }) },
      }] });
      emit({}, 'tool_calls');
    } else {
      if (note) emit({ role: 'assistant', reasoning_content: 'Synthetic public reasoning.' });
      emit({ role: 'assistant', content: note ? `NOTE: ${note}` : 'The ticket surface is ready.' });
      emit({}, 'stop');
    }
    response.end('data: [DONE]\n\n');
  } catch (error) {
    record({ kind: 'fixture-error', message: error.message });
    if (response.headersSent) response.destroy();
    else response.writeHead(422).end('Owned fixture rejected request');
  }
}).listen(Number(port), '127.0.0.1');
