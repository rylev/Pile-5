<template>
  <div class="game">
    <div class="table">
      <Piles :piles="piles" :pickPile="pickPile" />
      <div class="info">
        <div class="round">
          <div>Round: {{ round }}</div>
          <div>Score:{{ points }}</div>
        </div>
        <div v-if="pickPile" class="alert">Pick a tile!</div>
      </div>
    </div>
    <CardHand :cards="hand" :playedCard="playedCard" :sendPlayCard="sendPlayCard" />
    <button @click="restartGame" class="reset">Reset Game</button>
  </div>
</template>

<script>
import CardHand from './CardHand.vue'
import Piles from './Piles.vue'

export default {
  name: 'Game',
  components: {
    CardHand, Piles
  },
  props: {
    points: {
      type: Number,
      required: true,
    },
    round: {
      type: Number,
      required: true,
    },
    playedCard: {
      type: [Number, null],
      required: true,
    },
    piles: {
      type: Array,
      required: true,
    },
    hand: {
      type: Array,
      required: true,
    },
    pickPile: {
      type: [Function, null],
      required: true,
    },
    sendPlayCard: {
      type: Function,
      required: true,
    },
    restartGame: {
      type: Function,
      required: true,
    }
  },
}
</script>


<style scoped>
.game {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.table {
  width: 90%;
  display: flex;
  align-items: center;
}

.info {
  font-weight: bold;
  width: 20%;
  display: flex;
  justify-content: center;
  align-items: center;
  flex-direction: column;
}

.round {
  background-color: #38A3A5;
  width: 80px;
  padding: 10px;
  margin: 10px;
}

.reset {
  width: 160px;
  margin: 10px;
  display: inline-block;
  outline: none;
  cursor: pointer;
  font-size: 16px;
  line-height: 20px;
  font-weight: 600;
  border-radius: 8px;
  padding: 14px 24px;
  border: none;
  background: #57CC99;
  color: #111;
}

.alert {
  background-color: #f44336;
  color: white;
  font-weight: bold;
  padding: 20px;
  font-size: 2rem;
}
</style>