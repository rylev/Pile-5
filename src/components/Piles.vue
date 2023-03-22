<template>
    <div class="card-piles">
        <div v-for="(pile, pileIndex) in piles" :key="pileIndex" class="bundle">
            <div class="pile-mate" :class="{ changed: changed.includes(pileIndex) }">
                <div class="card-pile" @click="() => pickPile(pileIndex)" :class="{ hoverable: !!pickPile }">
                    <div v-for="(other, cardIndex) in pile.slice(0, pile.length - 1)" :key="cardIndex" class="under"
                        :style="{ position: 'absolute', 'z-index': cardIndex, top: `${(cardIndex * 5)}px` }">
                    </div>
                    <Card class="card"
                        :style="{ position: 'absolute', 'z-index': pile.length - 1, top: `${(pile.length - 1) * 5}px` }"
                        :key="pileIndex" :cardValue="pile[pile.length - 1]" />
                </div>
            </div>
            <div>
                <div>Cards: {{ pile.length }}</div>
                <div>Points: {{ pilePoints[pileIndex] }}</div>
            </div>
        </div>
    </div>
</template>
  
<script>
import Card from './Card.vue';
import { points } from '../points.js'
import { toRaw } from 'vue';

export default {
    props: {
        piles: {
            type: Array,
            required: true
        },
        pickPile: {
            type: [Function, null],
            required: true
        }
    },
    components: {
        Card
    },
    data() {
        return {
            changed: []
        }
    },
    computed: {
        pilePoints() {
            return this.piles.map(pile => {
                return pile.map(value => points(value)).reduce((acc, n) => acc + n);
            })
        }
    },
    watch: {
        piles(oldPiles, newPiles) {
            let changed = [];
            if (oldPiles && newPiles) {
                let o = toRaw(oldPiles);
                let n = toRaw(newPiles);
                for (let i = 0; i < 4; i++) {
                    if (!arraysEqual(o[i], n[i])) {
                        changed.push(i);
                    }
                }
            }
            this.changed = changed;
            setTimeout(() => {
                this.changed = [];
            }, 1000)
        }
    }
}
function arraysEqual(a, b) {
    if (a === b) return true;
    if (a.length !== b.length) return false;

    for (var i = 0; i < a.length; ++i) {
        if (a[i] !== b[i]) return false;
    }
    return true;
}
</script>
  
<style>
.card-piles {
    display: flex;
    justify-content: space-evenly;
    margin: 1rem;
    border: 1px solid #333;
    width: 80%;
    padding: 4px
}

.bundle {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    margin-top: 10px;
    margin-bottom: 10px;

    width: 25%;
    height: 220px;
}

.pile-mate {
    width: 90%;
    height: 100%;
    padding: 5px;
    transition: background-color 1s linear;
    border-radius: 5px;
}

.card-pile {
    width: 100%;
    height: 100%;
    position: relative;
}

.under {
    left: 0;
    right: 0;
    margin-left: auto;
    margin-right: auto;
    height: 100px;
    width: 100px;
    background-color: black;
    border: 1px solid #ccc;
    border-radius: 5px;
}

.card {
    left: 0;
    right: 0;
    margin: auto;
}

.hoverable {
    cursor: pointer;
}

.hover:hover {
    color: #333;
}

.changed {
    background-color: #38A3A5;
}
</style>
  