<template>
    <div class="card-picker">
        <div id="played-card">
            <Card :cardValue="playedCard" @click="unplayCard" />
        </div>
        <div class="cards">
            <template v-for="(card, index) in cards" :key="index">
                <Card v-if="selectedCard === card && !playedCard" class="selected" :cardValue="card"
                    @click="playCard(card)" />
                <Card v-else-if="playedCard !== card" class="unselected" :cardValue="card" @click="selectCard(card)" />
                <Card v-else class="unselected" :cardValue="null" @click="selectCard(card)" />
            </template>
        </div>
    </div>
</template>

<script>
import Card from './Card.vue';

export default {
    components: {
        Card,
    },
    props: {
        cards: {
            type: Array,
            required: true,
        },
    },
    data() {
        return {
            selectedCard: this.cards[0],
            playedCard: null
        };
    },
    methods: {
        selectCard(card) {
            this.selectedCard = card;
        },
        playCard(card) {
            this.playedCard = card;
        },
        unplayCard() {
            this.playedCard = null;
        }
    },
};
</script>

<style scoped>
.card-picker {
    width: 100%;
    display: flex;
    align-items: center;
    flex-direction: column;
}

.cards {
    margin: 10px;
    display: flex;
    justify-content: center;
    position: relative;
    width: 80%;
}

.card {
    margin: 1px;
}

.selected {
    transition: all 0.2s ease-in-out;
    transform: translateY(-10px);
}

.unselected {
    background-color: #999;
}

.unselected:not(.empty):hover {
    transform: translateY(-10px);
}
</style>
