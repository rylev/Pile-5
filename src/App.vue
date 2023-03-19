<template>
  <Piles :piles="piles" />
  <CardHand :cards="hand" />
</template>

<script>
import CardHand from './components/CardHand.vue'
import Piles from './components/Piles.vue'

export default {
  name: 'App',
  components: {
    CardHand, Piles
  },
  data() {
    return {
      hand: [15, 27, 39, 49, 55],
      piles: [[1, 2, 3, 4, 5], [2], [3], [4]]
    }
  }
}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
