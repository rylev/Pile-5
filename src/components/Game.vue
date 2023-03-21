<template>
  <div>Round: {{ round }}</div>
  <div v-if="pickPile">GOTTA PICK A PILE</div>
  <Piles :piles="piles" :pickPile="pickPile" />
  <CardHand :cards="hand" :playedCard="playedCard" :sendPlayCard="sendPlayCard" />
  <div>Score:{{ points }}</div>
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
    }
  },
}
</script>

