<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Support\Str;

class Exam extends Model
{
    use HasFactory;

    protected static function booted(): void
    {
        static::creating(function (Exam $exam): void {
            if (! filled($exam->token_global)) {
                $exam->token_global = static::generateUniqueToken();
            }
        });
    }

    protected $fillable = [
        'title',
        'subject',
        'class_room',
        'exam_url',
        'start_at',
        'end_at',
        'duration_minutes',
        'token_global',
        'is_active',
        'created_by',
    ];

    protected $casts = [
        'start_at' => 'datetime',
        'end_at' => 'datetime',
        'is_active' => 'boolean',
    ];

    public function creator(): BelongsTo
    {
        return $this->belongsTo(User::class, 'created_by');
    }

    public function sessions(): HasMany
    {
        return $this->hasMany(ExamSession::class);
    }

    public function launchTokens(): HasMany
    {
        return $this->hasMany(LaunchToken::class);
    }

    public static function generateUniqueToken(): string
    {
        do {
            $token = Str::upper(Str::random(6));
        } while (static::query()->where('token_global', $token)->exists());

        return $token;
    }
}
