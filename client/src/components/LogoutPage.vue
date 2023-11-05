<script setup>
setTimeout(() => {
    document.getElementById('logout_form').submit();
}, 1000);
</script>

<template>
    <div class="flex justify-center items-center bg-gray-950 h-screen">
        <h1 class="text-4xl text-white font-bold">You'll logged out.</h1>
        <form class="hidden" id="logout_form" action="https://localhost:8443/api/user/logout" method="post"></form>
    </div>
</template>