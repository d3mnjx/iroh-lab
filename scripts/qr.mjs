import { readFile } from 'node:fs/promises';
import QRCode from 'qrcode';
const url = (await readFile('runtime/pairing-url.txt', 'utf8')).trim();
await QRCode.toFile('runtime/pairing-qr.png', url, { width: 640, margin: 3, errorCorrectionLevel: 'L' });
console.log('Private QR code saved to runtime/pairing-qr.png. Do not commit or publish it.');
