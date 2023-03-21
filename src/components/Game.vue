<template>
  <div>Round: {{ round }}</div>
  <Piles :piles="piles" />
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
    sendPlayCard: {
      type: Function,
      required: true,
    }
  },
}
</script>

