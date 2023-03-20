<template>
  <div>Round: {{ state.round }}</div>
  <Piles :piles="state.piles" />
  <CardHand :cards="state.me.hand" />
  <div>Score:{{ state.me.points }}</div>
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

