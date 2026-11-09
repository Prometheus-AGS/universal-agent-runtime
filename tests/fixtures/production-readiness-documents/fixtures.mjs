// Deterministic document inputs. cobalt.png freezes the synthetic "FACT 73"
// lettering in Roboto Bold so the behavior runner needs no installed fonts.
import { readFileSync } from 'node:fs';

const text = 'The cobalt qualification coefficient is 73. Seal TEAL-7319.';
const crc32 = bytes => {
  let value = 0xffffffff;
  for (const byte of bytes) {
    value ^= byte;
    for (let bit = 0; bit < 8; bit++) value = (value >>> 1) ^ ((value & 1) ? 0xedb88320 : 0);
  }
  return (value ^ 0xffffffff) >>> 0;
};

// Stored ZIP entries with a fixed DOS epoch. Small OOXML/archive fixtures need
// neither external tools nor a newly installed package.
function zip(entries) {
  const local = [], central = [];
  let offset = 0;
  for (const [name, content] of entries) {
    const filename = Buffer.from(name), bytes = Buffer.from(content), checksum = crc32(bytes);
    const header = Buffer.alloc(30);
    header.writeUInt32LE(0x04034b50, 0); header.writeUInt16LE(20, 4);
    header.writeUInt16LE(33, 12); header.writeUInt32LE(checksum, 14);
    header.writeUInt32LE(bytes.length, 18); header.writeUInt32LE(bytes.length, 22);
    header.writeUInt16LE(filename.length, 26);
    local.push(header, filename, bytes);
    const index = Buffer.alloc(46);
    index.writeUInt32LE(0x02014b50, 0); index.writeUInt16LE(20, 4); index.writeUInt16LE(20, 6);
    index.writeUInt16LE(33, 14); index.writeUInt32LE(checksum, 16);
    index.writeUInt32LE(bytes.length, 20); index.writeUInt32LE(bytes.length, 24);
    index.writeUInt16LE(filename.length, 28); index.writeUInt32LE(offset, 42);
    central.push(index, filename);
    offset += header.length + filename.length + bytes.length;
  }
  const directory = Buffer.concat(central), end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0); end.writeUInt16LE(entries.length, 8);
  end.writeUInt16LE(entries.length, 10); end.writeUInt32LE(directory.length, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...local, directory, end]);
}

const types = main => `<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>${main}</Types>`;
const relationship = (type, target) => `<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/${type}" Target="${target}"/></Relationships>`;

export function documentFixtures() {
  const docx = zip([
    ['[Content_Types].xml', types('<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>')],
    ['_rels/.rels', relationship('officeDocument', 'word/document.xml')],
    ['word/document.xml', `<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>${text}</w:t></w:r></w:p></w:body></w:document>`],
  ]);
  const xlsx = zip([
    ['[Content_Types].xml', types('<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>')],
    ['_rels/.rels', relationship('officeDocument', 'xl/workbook.xml')],
    ['xl/workbook.xml', '<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Cobalt" sheetId="1" r:id="rId1"/></sheets></workbook>'],
    ['xl/_rels/workbook.xml.rels', relationship('worksheet', 'worksheets/sheet1.xml')],
    ['xl/worksheets/sheet1.xml', `<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>${text}</t></is></c></row></sheetData></worksheet>`],
  ]);
  return [
    { name: 'cobalt.docx', type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document', bytes: docx, expected: ['73', 'TEAL-7319'] },
    { name: 'cobalt.xlsx', type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet', bytes: xlsx, expected: ['73', 'TEAL-7319'] },
    { name: 'cobalt.eml', type: 'message/rfc822', bytes: Buffer.from(`From: fixture@example.invalid\r\nTo: owned@example.invalid\r\nSubject: Cobalt fact\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n${text}\r\n`), expected: ['73', 'TEAL-7319'] },
    { name: 'cobalt.zip', type: 'application/zip', bytes: zip([['cobalt.txt', text]]), expected: ['73', 'TEAL-7319'] },
    { name: 'cobalt.png', type: 'image/png', bytes: readFileSync(new URL('./cobalt.png', import.meta.url)), expected: ['73'], requires: 'configured OCR backend' },
  ];
}
