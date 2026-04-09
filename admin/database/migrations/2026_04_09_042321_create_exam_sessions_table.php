<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('exam_sessions', function (Blueprint $table) {
            $table->id();
            $table->foreignId('exam_id')->constrained('exams')->cascadeOnDelete();
            $table->string('display_name');
            $table->string('class_room');
            $table->string('client_type');
            $table->string('device_id')->nullable();
            $table->string('status')->default('active');
            $table->dateTime('started_at')->nullable();
            $table->dateTime('last_heartbeat_at')->nullable();
            $table->dateTime('ended_at')->nullable();
            $table->string('ip_address', 45)->nullable();
            $table->text('user_agent')->nullable();
            $table->timestamps();

            $table->index(['exam_id', 'status']);
            $table->index('last_heartbeat_at');
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('exam_sessions');
    }
};
