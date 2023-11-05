<script setup>

</script>

<template>
    <div class="flex flex-col gap-y-8 justify-center items-center bg-gray-950 h-screen">
        <img src="/src/assets/speech-bubble.png" class="w-48 h-48" alt="Chat logo">
        
        <h1 class="text-gray-100 text-4xl font-bold">Login</h1>
        <form class="flex flex-col gap-y-2 w-[50%]" action="https://localhost:8443/api/user/login" method="post">
            <input class="rounded-lg p-2 bg-gray-800 text-gray-100" type="text" name="username" id="username" placeholder="Username">
            <input class="rounded-lg p-2 bg-gray-800 text-gray-100" type="password" name="password" id="password" placeholder="Password">

            <input class="bg-blue-600 text-gray-200 font-bold px-4 py-2 rounded-lg" type="submit" value="Login">
            <div class="flex justify-between">
                <router-link class="text-blue-300 text-sm" to="/register">Register</router-link>
                <a class="text-blue-300 text-sm" onclick="alert('Not supported for now.');">Reset password</a>
            </div>
        </form>
    </div>
</template>