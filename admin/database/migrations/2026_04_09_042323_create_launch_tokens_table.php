<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('launch_tokens', function (Blueprint $table) {
            $table->id();
            $table->foreignId('exam_id')->constrained('exams')->cascadeOnDelete();
            $table->foreignId('exam_session_id')->nullable()->constrained('exam_sessions')->nullOnDelete();
            $table->string('token_hash', 64)->unique();
            $table->string('issued_for_client');
            $table->string('issued_for_ip', 45)->nullable();
            $table->json('payload_json')->nullable();
            $table->dateTime('expires_at');
            $table->dateTime('used_at')->nullable();
            $table->timestamps();

            $table->index(['exam_id', 'expires_at']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('launch_tokens');
    }
};
