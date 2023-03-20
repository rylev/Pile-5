<template>
  <div>Round: {{ state.round }}</div>
  <Piles :piles="piles" />
  <CardHand :cards="hand" />
  <div>Score:{{ state.score }}</div>
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
    state: {
      type: Object,
      required: true,
    },
  },
}
</script>

