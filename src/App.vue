<template>
  <CardHand :cards="cards" />
</template>

<script>
import CardHand from './components/CardHand.vue'

export default {
  name: 'App',
  components: {
    CardHand
  },
  data() {
    return {
      cards: [15, 27, 39, 49, 55]
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
