<template>
    <div class="card-piles">
        <div v-for="(pile, pileIndex) in piles" :key="pileIndex" class="bundle">
            <div class="card-pile" @click="() => pickPile(pileIndex)" :class="{ hoverable: !!pickPile }">
                <div v-for="(other, cardIndex) in pile.slice(0, pile.length - 1)" :key="cardIndex" class="under"
                    :style="{ position: 'absolute', 'z-index': cardIndex, top: `${(cardIndex * 5)}px` }">
                </div>
                <Card :style="{ position: 'absolute', 'z-index': pile.length - 1, top: `${(pile.length - 1) * 5}px` }"
                    :key="pileIndex" :cardValue="pile[pile.length - 1]" />
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
    computed: {
        pilePoints() {
            return this.piles.map(pile => {
                return pile.map(value => points(value)).reduce((acc, n) => acc + n);
            })
        }
    }
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

    height: 220px;
}

.card-pile {
    box-sizing: border-box;
    width: 100px;
    height: 100%;
    position: relative;
}

.under {
    height: 100px;
    width: 100px;
    background-color: black;
    border: 1px solid #ccc;
    border-radius: 5px;
}

.hoverable {
    cursor: pointer;
}

.hover:hover {
    color: #333;
}
</style>
  