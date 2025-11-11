export function decodeState(state: bigint) {
  // converts bitboard to array of numbers
  // if array[i] = (state << i) & 0xf;
  // array is a 16 element array
  const array: number[] = new Array(16);
  for (let i = 0; i < 16; i++) {
    array[i] = Number((state >> BigInt(4 * i)) & 0xfn);
  }
  console.log(array);
  return array;
}

export const CELL_BG_COLORS = [
  "#cdc1b4", // 0
  "#eee4da", // 2
  "#ede0c8", // 4
  "#f2b179", // 8
  "#f59563", // 16
  "#f67c5f", // 32
  "#f65e3b", // 64
  "#edcf72", // 128
  "#edc850", // 256
  "#edc53f", // 512
  "#edc22e", // 1024
  "#ecc43f", // 2048
  "#3c3a32", // 4096
  "#3c3a32", // 8192
  "#3c3a32", // 16384
  "#3c3a32", // 32768
  "#3c3a32", // 65536
];

export const CELL_TEXT_COLORS = [
  "#776e65", // 0
  "#776e65", // 2
  "#776e65", // 4
  "#f9f6f2", // 8
  "#f9f6f2", // 16
  "#f9f6f2", // 32
  "#f9f6f2", // 64
  "#f9f6f2", // 128
  "#f9f6f2", // 256
  "#f9f6f2", // 512
  "#f9f6f2", // 1024
  "#f9f6f2", // 2048
  "#f9f6f2", // 4096
  "#f9f6f2", // 8192
  "#f9f6f2", // 16384
  "#f9f6f2", // 32768
  "#f9f6f2", // 65536
];
