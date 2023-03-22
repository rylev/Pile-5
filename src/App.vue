<template>
  <template v-if="!userId">
    <TitleHeading />
    <div v-if="error" class="error"> Error: {{ error }}</div>
    <form @submit.prevent="handleJoin">
      <label for="my-input" id="label">Enter your name:</label>
      <input type="text" id="my-input" v-model="playerName">
      <button type="submit" class="join">Join</button>
    </form>
  </template>
  <NetworkedApp v-else :userId="userId" :authenticationFailed="authenicationFailed" />
</template>

<script>
import NetworkedApp from './components/NetworkedApp.vue'
import TitleHeading from './components/TitleHeading.vue'

export default {
  name: 'App',
  components: {
    NetworkedApp, TitleHeading
  },
  data() {
    return {
      userId: null,
      playerName: null,
      error: null
    }
  },
  methods: {
    authenicationFailed() {
      console.log("Authentication failed");
      localStorage.clear();
      this.userId = null;
      this.error = "Authentication failed!";
    },
    handleJoin() {
      console.log(`Request for ${this.playerName} to join the game`);
      fetch("/join", {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ name: this.playerName })
      })
        .then(response => {
          if (!response.ok) {
            throw new Error('Network response was not ok');
          }
          return response.json();
        })
        .then(json => {
          console.log('Success:', JSON.stringify(json));
          this.userId = json.user_id;
          localStorage.setItem("user_id", this.userId);
          localStorage.setItem("version", json.version);
        })
        .catch(error => {
          console.error('Error:', error);
        });

    }
  },
  mounted() {
    const version = localStorage.getItem("version");
    this.userId = localStorage.getItem("user_id");
    console.log("UserId: ", this.userId, " Version: ", version);
    if (version && this.userId) {
      fetch("/version").then(r => r.text()).then(v => {
        if (v !== version) {
          localStorage.clear();
          this.userId = null;
        }
      })
    }
  },
}
</script>

<style scoped>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
}

.join {
  display: inline-block;
  outline: none;
  cursor: pointer;
  font-size: 16px;
  line-height: 20px;
  font-weight: 600;
  width: 100px;
  border-radius: 8px;
  padding: 14px;
  border: none;
  background: #57CC99;
  color: #111;
  margin: 10px;
}

input,
label {
  display: block;
}

#my-input {
  margin: 5px auto;
  font-size: 1.5rem;
  text-align: center;
}

#label {
  font-size: 2rem;
}

.error {
  color: red;
  background-color: #f8d7da;
  width: 40%;
  margin: 0 auto;
  border-radius: 5px;
}
</style>
