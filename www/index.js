import * as rust from "wasm_pubsub";
import {
    Universe,
    Cell
} from "wasm_pubsub";
import {
    memory
} from "../pkg/wasm_pubsub_bg.wasm";

const CELL_SIZE = 5; // px
const MAP_SIZE = 128;
const GRID_COLOR = "#CCCCCC";
const BLACK_COLOR = "#000000";
const YELLOW_COLOR = "#FFFF00";

async function subscribe() {
    let count = 0;
    let time = "";

    while (true) {
        await rust.subscribe(time, "global").then((data) => {
            time = data.t.t;
            for (var m_index in data.m) {
                if(data.m[m_index].d.tick){
                    requestAnimationFrame(renderLoop);
                    count = count + 1;
                    document.getElementById("p1").innerHTML = "Local Universe Ticked " + count + " times";
                }else{
                    let cellArray = data.m[m_index].d.cells.split(' ');
                    
                    for(let i = 0; i < cellArray.length; i++){
                        let row = parseInt(cellArray[i]);
                        let col = parseInt(cellArray[++i]);
                        let alive = (cellArray[++i] == "true");

                        universe.set_cell(row, col, alive);
                        drawACell(row, col, alive);
                    }
                }  
            }
        });
    }
}

const universe = Universe.new(MAP_SIZE);
const width = universe.width();
const height = universe.height();
let userAlive = false;

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById("game-of-life-canvas");
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;

const ctx = canvas.getContext('2d');

const renderLoop = () => {
    universe.tick();
    drawCells();
};
const drawGrid = () => {
    ctx.beginPath();
    ctx.strokeStyle = GRID_COLOR;

    // Vertical lines.
    for (let i = 0; i <= width; i++) {
        ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
        ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
    }

    // Horizontal lines.
    for (let j = 0; j <= height; j++) {
        ctx.moveTo(0, j * (CELL_SIZE + 1) + 1);
        ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
    }

    ctx.stroke();
};
const getIndex = (row, column) => {
    return row * width + column;
};

const drawACell = (row, col, alive) => {

    ctx.beginPath();
    ctx.fillStyle = alive ?
                    YELLOW_COLOR:
                    BLACK_COLOR;

    ctx.fillRect(
        col * (CELL_SIZE + 1) + 1,
        row * (CELL_SIZE + 1) + 1,
        CELL_SIZE,
        CELL_SIZE
    );
    ctx.stroke();
};
const drawCells = () => {
    
    const cellsPtr = universe.cells();
    const cells = new Uint8Array(memory.buffer, cellsPtr, width * height);

    ctx.beginPath();
    
    ctx.fillStyle = YELLOW_COLOR;
    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            const idx = getIndex(row, col);
            if (cells[idx] !== Cell.Alive) {
            continue;
            }

            ctx.fillRect(
            col * (CELL_SIZE + 1) + 1,
            row * (CELL_SIZE + 1) + 1,
            CELL_SIZE,
            CELL_SIZE
            );
        }
    }

    ctx.fillStyle = BLACK_COLOR;
    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
            const idx = getIndex(row, col);
            if (cells[idx] !== Cell.Dead) {
            continue;
            }

            ctx.fillRect(
            col * (CELL_SIZE + 1) + 1,
            row * (CELL_SIZE + 1) + 1,
            CELL_SIZE,
            CELL_SIZE
            );
        }
    }
    ctx.stroke();
};
const getRCofMouse = (cX, cY) => {
    const boundingRect = canvas.getBoundingClientRect();
    
    const scaleX = canvas.width / boundingRect.width;
    const scaleY = canvas.height / boundingRect.height;

    const canvasLeft = (cX - boundingRect.left) * scaleX;
    const canvasTop = (cY - boundingRect.top) * scaleY;
    
    const row = Math.min(Math.floor(canvasTop / (CELL_SIZE + 1)), height - 1);
    const col = Math.min(Math.floor(canvasLeft / (CELL_SIZE + 1)), width - 1);
    return {
        row,
        col,
    }
}
let squares = new Map();
let mDown = false;
let lastSquare = "";
canvas.addEventListener("mousedown", event => {
    var {row, col} = getRCofMouse(event.clientX, event.clientY);
    let key = row.toString() + ' ' + col.toString();
    squares.set(key, userAlive);
    lastSquare = key;
    mDown = true;

    drawACell(row, col, userAlive);
    universe.set_cell(row, col, userAlive);
});
canvas.addEventListener("mousemove", event => {
    if(mDown){
        var {row, col} = getRCofMouse(event.clientX, event.clientY);
        let key = row.toString() + ' ' + col.toString();
        if(lastSquare != key){
            squares.set(key, userAlive)
            lastSquare = key;
            universe.set_cell(row, col, userAlive);
            drawACell(row, col, userAlive);
        }
    }

});
canvas.addEventListener("mouseup", event => {
    mDown = false;
    
    let sArray = new Array();
    let rcString = "";
    squares.forEach((value, key, map) => {
        rcString += key + " " + value.toString() + " ";
    });
    rust.publish({tick: false, cells: rcString.slice(0, -1)}, "global").then((data) => {
        console.log(data.sent);
    });
    squares = new Map();
    lastSquare = "";
});

drawGrid();
drawCells();
requestAnimationFrame(renderLoop);

document.getElementById("tick").onclick =  () => {
    rust.publish({tick: true, cells: ""}, "global").then((data) => {
        console.log(data.sent);
    });
};
document.getElementById("color_btn").onclick =  () => {
    userAlive = !userAlive;

    if(userAlive){
        document.getElementById("user_color").style.backgroundColor = YELLOW_COLOR;
        document.getElementById("color_btn").innerHTML = "Choose Black";
    }else{
        document.getElementById("user_color").style.backgroundColor = BLACK_COLOR;
        document.getElementById("color_btn").innerHTML = "Choose Yellow";
    }
};
subscribe();