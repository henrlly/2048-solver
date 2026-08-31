<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { CELL_BG_COLORS, CELL_TEXT_COLORS, decodeState } from "./utils";

    let state: bigint = 0n;
    let table: number[] = decodeState(state);
    let isSolving: boolean = false;
    let gameOver: boolean = false;
    let score: number = 0;

    async function move(direction: string, calledFromSolver: boolean = false) {
        // We can't pass BigInt directly,
        // because Tauri's invoke only supports JSON-compatible types,
        // so we convert state to string.
        isSolving = calledFromSolver;
        const [stateStr, additional_score]: [string, number] = await invoke(
            `make_move`,
            {
                state: state.toString(),
                direction,
            },
        );
        state = BigInt(stateStr);
        table = decodeState(state);
        if (additional_score !== -1) {
            score += additional_score;
        }
        if (additional_score === -1) {
            gameOver = true;
            isSolving = false;
        }
    }

    async function resetBoard() {
        state = 0n | (1n << BigInt(Math.floor(Math.random() * 16) * 4));
        table = decodeState(state);
        isSolving = false;
        gameOver = false;
        score = 0;
    }

    onMount(resetBoard);

    async function startSolving() {
        while (isSolving && !gameOver) {
            const [bestMove, score]: [string, number] = await invoke(
                "find_best_move",
                {
                    state: state.toString(),
                },
            );
            if (score === 0) {
                isSolving = false;
                gameOver = true;
                break;
            }
            await move(bestMove, true);
        }
    }

    async function onKeyDown(event: KeyboardEvent) {
        if (event.key === "ArrowUp" || event.key === "w") {
            move("up");
        } else if (event.key === "ArrowDown" || event.key === "s") {
            move("down");
        } else if (event.key === "ArrowLeft" || event.key === "a") {
            move("left");
        } else if (event.key === "ArrowRight" || event.key === "d") {
            move("right");
        } else if (event.key === "r") {
            isSolving = !isSolving;
            startSolving();
        } else if (event.key === "q") {
            resetBoard();
        }
    }
</script>

<div>
    <p>Press R to solve; Q to reset; WASD/Arrow keys to move</p>
    <!-- add commas -->
    <p>Score: {score.toLocaleString('en-US')}</p>
    <p>{isSolving ? "Solving..." : ""}</p>
    <p>{gameOver ? "Game Over!" : ""}</p>
    <div
        style="display: grid; grid-template-columns: repeat(4, 50px); grid-template-rows: repeat(4, 50px); gap: 5px;"
    >
        {#each table as cell}
            <div
                style="width: 50px; height: 50px; background-color: {CELL_BG_COLORS[
                    cell
                ]}; color: {CELL_TEXT_COLORS[
                    cell
                ]}; display: flex; align-items: center; justify-content: center; border: 1px solid black;"
            >
                {cell === 0 ? "" : 2 ** cell}
            </div>
        {/each}
    </div>
</div>

<svelte:window on:keydown={onKeyDown} />
