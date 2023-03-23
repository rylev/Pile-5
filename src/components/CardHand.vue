<template>
    <div class="card-picker">
        <div id="played-card">
            <Card :cardValue="playedCard" />
        </div>
        <div class="cards">
            <template v-if="!playedCard">
                <template v-for="(card, index) in cards" :key="index">
                    <Card v-if="selectedCard === card" class="selected" :cardValue="card" @click="playCard(card)" />
                    <Card v-else class="unselected" :inert="true" :cardValue="card" @click="selectCard(card)" />
                </template>
            </template>
            <template v-else>
                <template v-for="(card, index) in cards" :key="index">
                    <Card v-if="playedCard === card" :cardValue="null" />
                    <Card v-else class="unselected inert" :inert="true" :cardValue="card" />
                </template>
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
        playedCard: {
            type: [Number, null],
            required: true
        },
        sendPlayCard: {
            type: Function,
            required: true
        },
    },
    data() {
        return {
            selectedCard: this.cards[0],
        };
    },
    methods: {
        selectCard(card) {
            this.selectedCard = card;
        },
        playCard(card) {
            this.sendPlayCard(card)
        },
    },
};
</script>

<style scoped>
.card-picker {
    width: 90%;
    border-radius: 5px;
    display: flex;
    align-items: center;
    flex-direction: column;
    background: #38A3A5;
    padding: 20px;
}

.cards {
    margin: 10px;
    display: flex;
    justify-content: center;
    position: relative;
    width: 90%;
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

.unselected:not(.inert):hover {
    transform: translateY(-10px);
}
</style>
