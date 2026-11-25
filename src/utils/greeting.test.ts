/**
 * Pruebas unitarias para el módulo `greeting`.
 *
 * Lógica: Verifica de forma determinista con `node:test` que el texto devuelto
 * por `getHelloWorld` coincide exactamente con "Hello World".
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import { getHelloWorld } from './greeting';

test('getHelloWorld returns exact Hello World text', () => {
  assert.equal(getHelloWorld(), 'Hello World');
});
